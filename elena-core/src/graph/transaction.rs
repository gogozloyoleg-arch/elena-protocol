//! Транзакции, алерты и локальный граф.

use crate::crypto::{hash_512, generate_nonce, current_timestamp_ms, CryptoError, KeyPair};
use hex;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::{HashMap, HashSet};

/// ID транзакции (SHA3-512)
pub type TxId = [u8; 64];
/// Якорь (SHA3-512)
pub type Anchor = [u8; 64];

fn serialize_bytes64<S: Serializer>(v: &[u8; 64], s: S) -> Result<S::Ok, S::Error> {
    s.serialize_bytes(v)
}
fn deserialize_bytes64<'de, D: Deserializer<'de>>(d: D) -> Result<[u8; 64], D::Error> {
    let v: Vec<u8> = Deserialize::deserialize(d)?;
    if v.len() != 64 {
        return Err(serde::de::Error::custom("expected 64 bytes"));
    }
    let mut a = [0u8; 64];
    a.copy_from_slice(&v);
    Ok(a)
}
fn serialize_parents<S: Serializer>(v: &[TxId], s: S) -> Result<S::Ok, S::Error> {
    use serde::ser::SerializeSeq;
    let mut seq = s.serialize_seq(Some(v.len()))?;
    for item in v {
        seq.serialize_element(&item[..])?;
    }
    seq.end()
}
fn deserialize_parents<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<TxId>, D::Error> {
    let v: Vec<Vec<u8>> = Deserialize::deserialize(d)?;
    let mut out = Vec::with_capacity(v.len());
    for bytes in v {
        if bytes.len() != 64 {
            return Err(serde::de::Error::custom("expected 64 bytes per parent"));
        }
        let mut a = [0u8; 64];
        a.copy_from_slice(&bytes);
        out.push(a);
    }
    Ok(out)
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum TransactionType {
    Payment,
    Alert,
    Stake,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Transaction {
    #[serde(serialize_with = "serialize_bytes64", deserialize_with = "deserialize_bytes64")]
    pub id: TxId,
    pub tx_type: TransactionType,
    pub from: Vec<u8>,
    pub to: Vec<u8>,
    pub amount: u64,
    pub nonce: u64,
    #[serde(serialize_with = "serialize_bytes64", deserialize_with = "deserialize_bytes64")]
    pub anchor: Anchor,
    #[serde(serialize_with = "serialize_parents", deserialize_with = "deserialize_parents")]
    pub parents: Vec<TxId>,
    pub timestamp: u64,
    pub signature: Vec<u8>,
    pub is_chaff: bool,
    /// Комиссия (микро-ELENA); при сериализации старых данных по умолчанию 0
    #[serde(default)]
    pub fee: u64,
}

impl Transaction {
    /// Данные для подписи (детерминированный порядок полей)
    pub fn content_to_sign(&self) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(&self.from);
        data.extend_from_slice(&self.to);
        data.extend_from_slice(&self.amount.to_le_bytes());
        data.extend_from_slice(&self.nonce.to_le_bytes());
        data.extend_from_slice(&self.anchor);
        for p in &self.parents {
            data.extend_from_slice(p);
        }
        data.extend_from_slice(&self.timestamp.to_le_bytes());
        data.extend_from_slice(&self.fee.to_le_bytes());
        data
    }

    pub fn compute_id(&self) -> TxId {
        hash_512(&self.content_to_sign())
    }

    /// Создаёт транзакцию и подписывает её. `fee` — комиссия в микро-ELENA.
    pub fn new(
        from: Vec<u8>,
        to: Vec<u8>,
        amount: u64,
        anchor: Anchor,
        parents: Vec<TxId>,
        fee: u64,
        keypair: &KeyPair,
    ) -> Result<Self, CryptoError> {
        let nonce = generate_nonce();
        let timestamp = current_timestamp_ms();
        let mut tx = Self {
            id: [0; 64],
            tx_type: TransactionType::Payment,
            from,
            to,
            amount,
            nonce,
            anchor,
            parents,
            timestamp,
            signature: vec![],
            is_chaff: false,
            fee,
        };
        tx.id = tx.compute_id();
        let content = tx.content_to_sign();
        tx.signature = keypair.sign(&content)?;
        Ok(tx)
    }

    pub fn verify_signature(&self, public_key: &[u8]) -> bool {
        crate::crypto::PublicKeyBytes::from_bytes(public_key)
            .and_then(|pk| pk.verify(&self.content_to_sign(), &self.signature))
            .unwrap_or(false)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Alert {
    #[serde(serialize_with = "serialize_bytes64", deserialize_with = "deserialize_bytes64")]
    pub id: TxId,
    #[serde(serialize_with = "serialize_bytes64", deserialize_with = "deserialize_bytes64")]
    pub conflicting_tx1: TxId,
    #[serde(serialize_with = "serialize_bytes64", deserialize_with = "deserialize_bytes64")]
    pub conflicting_tx2: TxId,
    #[serde(serialize_with = "serialize_bytes64", deserialize_with = "deserialize_bytes64")]
    pub anchor: Anchor,
    pub discovered_by: Vec<u8>,
    pub timestamp: u64,
    pub propagation_count: u32,
}

/// Снимок графа для сохранения на диск
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GraphSnapshot {
    pub transactions: Vec<Transaction>,
    pub alerts: Vec<Alert>,
}

/// Локальный граф узла
pub struct LocalGraph {
    pub transactions: HashMap<TxId, Transaction>,
    by_sender: HashMap<Vec<u8>, HashSet<TxId>>,
    by_anchor: HashMap<Anchor, HashSet<TxId>>,
    pub alerts: HashMap<TxId, Alert>,
}

impl LocalGraph {
    pub fn new() -> Self {
        Self {
            transactions: HashMap::new(),
            by_sender: HashMap::new(),
            by_anchor: HashMap::new(),
            alerts: HashMap::new(),
        }
    }

    /// Восстановить граф из снимка (индексы by_sender, by_anchor пересобираются)
    pub fn from_snapshot(snapshot: GraphSnapshot) -> Self {
        let mut graph = Self::new();
        for tx in snapshot.transactions {
            let tx_id = tx.id;
            graph.transactions.insert(tx_id, tx.clone());
            graph.by_sender.entry(tx.from.clone()).or_default().insert(tx_id);
            graph.by_anchor.entry(tx.anchor).or_default().insert(tx_id);
        }
        for alert in snapshot.alerts {
            graph.alerts.insert(alert.id, alert);
        }
        graph
    }

    pub fn to_snapshot(&self) -> GraphSnapshot {
        GraphSnapshot {
            transactions: self.transactions.values().cloned().collect(),
            alerts: self.alerts.values().cloned().collect(),
        }
    }

    /// Сохранить граф в JSON-файл
    pub fn save_to_path<P: AsRef<std::path::Path>>(
        &self,
        path: P,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(parent) = path.as_ref().parent() {
            std::fs::create_dir_all(parent)?;
        }
        let f = std::fs::File::create(path)?;
        serde_json::to_writer_pretty(f, &self.to_snapshot())?;
        Ok(())
    }

    /// Загрузить граф из JSON-файла; если файла нет — вернуть пустой граф
    pub fn load_from_path<P: AsRef<std::path::Path>>(
        path: P,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let path = path.as_ref();
        if !path.exists() {
            return Ok(Self::new());
        }
        let data = std::fs::read_to_string(path)?;
        let snapshot: GraphSnapshot = serde_json::from_str(&data)?;
        Ok(Self::from_snapshot(snapshot))
    }

    /// Добавляет транзакцию. Err при коллизии по якорю (один отправитель, один anchor, разные получатели).
    pub fn add_transaction(&mut self, tx: Transaction) -> Result<(), String> {
        let tx_id = tx.id;
        if let Some(existing) = self.by_anchor.get(&tx.anchor) {
            for &eid in existing {
                if eid != tx_id {
                    if let Some(other) = self.transactions.get(&eid) {
                        if other.from == tx.from && other.to != tx.to {
                            return Err("Collision detected".into());
                        }
                    }
                }
            }
        }
        self.transactions.insert(tx_id, tx.clone());
        self.by_sender
            .entry(tx.from.clone())
            .or_default()
            .insert(tx_id);
        self.by_anchor.entry(tx.anchor).or_default().insert(tx_id);
        Ok(())
    }

    pub fn find_collisions(&self, anchor: &Anchor) -> Vec<&Transaction> {
        self.by_anchor
            .get(anchor)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.transactions.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn get_confidence(&self, tx_id: &TxId) -> f64 {
        let mut confidence = 0.5;
        if let Some(_tx) = self.transactions.get(tx_id) {
            let refs = self
                .transactions
                .values()
                .filter(|t| t.parents.iter().any(|p| p == tx_id))
                .count();
            confidence += (refs as f64) * 0.1;
        }
        confidence.min(1.0)
    }

    /// Последние до `limit` ID транзакций от отправителя (порядок не гарантируется).
    pub fn recent_tx_ids_for_sender(&self, sender: &[u8], limit: usize) -> Vec<TxId> {
        self.by_sender
            .get(sender)
            .map(|ids| ids.iter().take(limit).copied().collect())
            .unwrap_or_default()
    }

    /// Последние транзакции по времени (по убыванию timestamp), не более limit.
    pub fn recent_transactions(&self, limit: usize) -> Vec<RecentTxItem> {
        let mut txs: Vec<_> = self
            .transactions
            .values()
            .map(|t| RecentTxItem {
                id: hex::encode(t.id),
                from: hex::encode(&t.from),
                to: hex::encode(&t.to),
                amount: t.amount,
                timestamp: t.timestamp,
            })
            .collect();
        txs.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        txs.into_iter().take(limit).collect()
    }
}

/// Элемент списка последних транзакций (для API).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RecentTxItem {
    pub id: String,
    pub from: String,
    pub to: String,
    pub amount: u64,
    pub timestamp: u64,
}

impl Default for LocalGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_id() {
        let kp = KeyPair::generate();
        let from = kp.public_key().to_vec();
        let to = vec![1u8; 32];
        let tx = Transaction::new(from, to, 1000, [0; 64], vec![], 0, &kp).unwrap();
        assert_eq!(tx.amount, 1000);
        assert!(tx.verify_signature(tx.from.as_slice()));
    }

    #[test]
    fn test_graph_collision() {
        let kp = KeyPair::generate();
        let from = kp.public_key().to_vec();
        let mut graph = LocalGraph::new();
        let t1 = Transaction::new(from.clone(), vec![2; 32], 100, [0; 64], vec![], 0, &kp).unwrap();
        let t2 = Transaction::new(from, vec![3; 32], 100, [0; 64], vec![], 0, &kp).unwrap();
        graph.add_transaction(t1).unwrap();
        assert!(graph.add_transaction(t2).is_err());
    }

    #[test]
    fn test_graph_snapshot_roundtrip() {
        let kp = KeyPair::generate();
        let from = kp.public_key().to_vec();
        let mut graph = LocalGraph::new();
        let t1 = Transaction::new(from.clone(), vec![2; 32], 100, [0; 64], vec![], 0, &kp).unwrap();
        graph.add_transaction(t1).unwrap();
        let snapshot = graph.to_snapshot();
        let restored = LocalGraph::from_snapshot(snapshot);
        assert_eq!(restored.transactions.len(), 1);
        assert_eq!(restored.recent_tx_ids_for_sender(&from, 5).len(), 1);
    }
}
