"""
Имитация пост-квантовой криптографии для симуляции сети Елена.
Используются хеши и подписи без реальных алгоритмов Dilithium/SHA3 — только поведение.
"""

import hashlib
import secrets
from typing import List, Tuple

# Хранилище для проверки подписей (в симуляции: data_hash -> valid)
_signature_store: dict = {}


def compute_anchor(
    balance: float,
    last_txs: List[str],
    nonce: int,
    timestamp: float,
) -> str:
    """
    Вычисляет якорь транзакции (имитация SHA3-512).
    anchor = hash(balance | last_tx1 | last_tx2 | nonce | timestamp)
    """
    parts = [
        str(balance),
        (last_txs[0] if len(last_txs) > 0 else ""),
        (last_txs[1] if len(last_txs) > 1 else ""),
        str(nonce),
        str(timestamp),
    ]
    payload = "|".join(parts)
    # SHA3-512 имитация через SHA-512 (для симуляции достаточно)
    return hashlib.sha512(payload.encode("utf-8")).hexdigest()


def _data_hash(data: str) -> str:
    """Хеш данных для подписи."""
    return hashlib.sha512(data.encode("utf-8")).hexdigest()


def generate_keypair() -> Tuple[str, str]:
    """Генерирует пару ключей (имитация для симуляции)."""
    private_key = secrets.token_hex(32)
    public_key = hashlib.sha256(private_key.encode("utf-8")).hexdigest()
    return public_key, private_key


def sign_data(data: str, private_key: str) -> bytes:
    """
    Имитация Dilithium-подписи: подпись = HMAC(priv, hash(data)).
    В симуляции достаточно детерминированного значения.
    """
    payload = private_key + _data_hash(data)
    sig_hash = hashlib.sha512(payload.encode("utf-8")).digest()
    public_key = hashlib.sha256(private_key.encode("utf-8")).hexdigest()
    _signature_store[( _data_hash(data), public_key)] = sig_hash
    return sig_hash


def verify_signature(data: str, signature: bytes, public_key: str) -> bool:
    """
    Имитация проверки подписи Dilithium.
    В симуляции проверяем, что подпись была создана для этих данных и ключа.
    """
    key = (_data_hash(data), public_key)
    if key not in _signature_store:
        return False
    return _signature_store[key] == signature


def tx_content_hash(
    from_id: str,
    to_id: str,
    amount: float,
    nonce: int,
    anchor: str,
    parents: List[str],
    timestamp: float,
) -> str:
    """Хеш содержимого транзакции для id и верификации."""
    parts = [from_id, to_id, str(amount), str(nonce), anchor]
    parts.extend(parents[:5])
    parts.append(str(timestamp))
    payload = "|".join(parts)
    return hashlib.sha512(payload.encode("utf-8")).hexdigest()
