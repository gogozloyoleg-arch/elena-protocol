//! Эхо-локация: детектор коллизий и уверенность в транзакциях.

use crate::graph::{Anchor, TxId};
use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

pub struct EchoConfig {
    pub confidence_threshold: f64,
    pub alert_priority_multiplier: u32,
    pub max_propagation_steps: u32,
}

pub struct CollisionDetector {
    anchors: HashMap<Anchor, HashSet<TxId>>,
    anchor_first_seen: HashMap<Anchor, Instant>,
    pub config: EchoConfig,
}

impl CollisionDetector {
    pub fn new(config: EchoConfig) -> Self {
        Self {
            anchors: HashMap::new(),
            anchor_first_seen: HashMap::new(),
            config,
        }
    }

    /// Возвращает список конфликтующих tx_id при коллизии по якорю.
    pub fn check_transaction(&mut self, tx_id: TxId, anchor: Anchor) -> Option<Vec<TxId>> {
        let entry = self.anchors.entry(anchor).or_insert_with(HashSet::new);
        if entry.is_empty() {
            self.anchor_first_seen.insert(anchor, Instant::now());
            entry.insert(tx_id);
            None
        } else {
            entry.insert(tx_id);
            Some(entry.iter().copied().collect())
        }
    }

    pub fn time_since_first_seen(&self, anchor: &Anchor) -> Option<Duration> {
        self.anchor_first_seen.get(anchor).map(Instant::elapsed)
    }
}

pub struct EchoLocator {
    confidence: HashMap<TxId, f64>,
    references: HashMap<TxId, HashSet<TxId>>,
}

impl EchoLocator {
    pub fn new() -> Self {
        Self {
            confidence: HashMap::new(),
            references: HashMap::new(),
        }
    }

    pub fn add_reference(&mut self, from: TxId, to: TxId) {
        self.references.entry(from).or_default().insert(to);
        self.update_confidence(&to);
    }

    fn update_confidence(&mut self, tx_id: &TxId) {
        let total: f64 = self
            .references
            .get(tx_id)
            .map(|refs: &HashSet<TxId>| refs.len() as f64 * 0.1)
            .unwrap_or(0.0);
        let c: f64 = 0.5 + total;
        self.confidence.insert(*tx_id, c.min(1.0));
    }

    pub fn is_final(&self, tx_id: &TxId, threshold: f64) -> bool {
        self.confidence.get(tx_id).copied().unwrap_or(0.0) >= threshold
    }

    pub fn atmospheric_pressure(&self, tx_id: &TxId) -> f64 {
        let mut visited = HashSet::new();
        let mut stack = vec![*tx_id];
        let mut depth = 0usize;
        while let Some(current) = stack.pop() {
            if visited.insert(current) {
                depth += 1;
                if let Some(refs) = self.references.get(&current) {
                    stack.extend(refs.iter().copied());
                }
            }
        }
        depth as f64 / 100.0
    }
}

impl Default for EchoLocator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collision_detection() {
        let config = EchoConfig {
            confidence_threshold: 0.99,
            alert_priority_multiplier: 10,
            max_propagation_steps: 5,
        };
        let mut d = CollisionDetector::new(config);
        let anchor = [0u8; 64];
        let tx1 = [1u8; 64];
        let tx2 = [2u8; 64];
        assert!(d.check_transaction(tx1, anchor).is_none());
        let c = d.check_transaction(tx2, anchor).unwrap();
        assert_eq!(c.len(), 2);
        assert!(c.contains(&tx1));
        assert!(c.contains(&tx2));
    }
}
