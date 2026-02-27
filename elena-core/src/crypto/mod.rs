//! Пост-квантовая криптография для сети «Елена».
//! CRYSTALS-Dilithium для подписей, SHA3-512 для хешей.

use pqcrypto_dilithium::dilithium3::{keypair, open, sign, PublicKey, SecretKey, SignedMessage};
use pqcrypto_traits::sign::{PublicKey as _, SecretKey as _, SignedMessage as _};
use sha3::{Digest, Sha3_512};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

/// Размер подписи Dilithium3
pub const SIGNATURE_SIZE: usize = 2420;
/// Размер публичного ключа
pub const PUBLIC_KEY_SIZE: usize = 1952;
/// Размер секретного ключа
pub const SECRET_KEY_SIZE: usize = 4032;

/// Криптографическая ошибка
#[derive(Debug, thiserror::Error)]
pub enum CryptoError {
    #[error("Ошибка подписи: {0}")]
    SignatureError(String),
    #[error("Ошибка верификации: {0}")]
    VerificationError(String),
    #[error("Неверный формат ключа")]
    InvalidKeyFormat,
}

/// Пара ключей Dilithium3
#[derive(Clone, Serialize, Deserialize)]
pub struct KeyPair {
    public_key: Vec<u8>,
    secret_key: Vec<u8>,
}

impl KeyPair {
    /// Генерирует новую пару ключей
    pub fn generate() -> Self {
        let (pk, sk) = keypair();
        Self {
            public_key: pk.as_bytes().to_vec(),
            secret_key: sk.as_bytes().to_vec(),
        }
    }

    /// Подписывает сообщение (возвращает полное SignedMessage для хранения)
    pub fn sign(&self, message: &[u8]) -> Result<Vec<u8>, CryptoError> {
        let sk = SecretKey::from_bytes(&self.secret_key)
            .map_err(|e| CryptoError::SignatureError(e.to_string()))?;
        let sm = sign(message, &sk);
        Ok(sm.as_bytes().to_vec())
    }

    /// Публичный ключ (байты)
    pub fn public_key(&self) -> &[u8] {
        &self.public_key
    }

    /// Секретный ключ (байты), для подписи транзакций
    pub fn secret_key(&self) -> &[u8] {
        &self.secret_key
    }

    /// Сериализация в байты (bincode)
    pub fn to_bytes(&self) -> Result<Vec<u8>, CryptoError> {
        bincode::serialize(self).map_err(|e| CryptoError::SignatureError(e.to_string()))
    }

    /// Десериализация из байтов (bincode)
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, CryptoError> {
        bincode::deserialize(bytes).map_err(|_| CryptoError::InvalidKeyFormat)
    }

    /// Сохранить ключи в файл (data_dir/wallets/<name>.key)
    pub fn save_to_path<P: AsRef<Path>>(&self, path: P) -> Result<(), CryptoError> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| CryptoError::SignatureError(e.to_string()))?;
        }
        let data = self.to_bytes()?;
        std::fs::write(path, data).map_err(|e| CryptoError::SignatureError(e.to_string()))?;
        Ok(())
    }

    /// Загрузить ключи из файла
    pub fn load_from_path<P: AsRef<Path>>(path: P) -> Result<Self, CryptoError> {
        let data = std::fs::read(path).map_err(|e| CryptoError::SignatureError(e.to_string()))?;
        Self::from_bytes(&data)
    }
}

/// Публичный ключ для верификации
#[derive(Clone, Serialize, Deserialize)]
pub struct PublicKeyBytes(pub Vec<u8>);

impl PublicKeyBytes {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, CryptoError> {
        PublicKey::from_bytes(bytes).map_err(|_| CryptoError::InvalidKeyFormat)?;
        Ok(Self(bytes.to_vec()))
    }

    /// Верифицирует подпись (signature = полный SignedMessage)
    pub fn verify(&self, message: &[u8], signature: &[u8]) -> Result<bool, CryptoError> {
        let pk = PublicKey::from_bytes(&self.0)
            .map_err(|e| CryptoError::VerificationError(e.to_string()))?;
        let sm = SignedMessage::from_bytes(signature)
            .map_err(|_| CryptoError::VerificationError("invalid signature".into()))?;
        match open(&sm, &pk) {
            Ok(opened) => Ok(opened == message),
            Err(_) => Ok(false),
        }
    }
}

/// SHA3-512 хеш
pub fn hash_512(data: &[u8]) -> [u8; 64] {
    let mut hasher = Sha3_512::new();
    hasher.update(data);
    hasher.finalize().into()
}

/// Криптографически безопасный nonce
pub fn generate_nonce() -> u64 {
    let mut buf = [0u8; 8];
    rand::thread_rng().fill_bytes(&mut buf);
    u64::from_le_bytes(buf)
}

/// Текущая метка времени (мс)
pub fn current_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_verify() {
        let keypair = KeyPair::generate();
        let message = b"Hello, Elena!";
        let signature = keypair.sign(message).unwrap();
        let pk = PublicKeyBytes::from_bytes(keypair.public_key()).unwrap();
        assert!(pk.verify(message, &signature).unwrap());
    }

    #[test]
    fn test_hash() {
        let h = hash_512(b"test");
        assert_eq!(h.len(), 64);
    }
}
