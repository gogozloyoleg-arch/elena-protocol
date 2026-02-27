//! Узел сети «Елена»: баланс, граф, консенсус, P2P, локальный RPC.

use crate::consensus::EchoLocator;
use crate::crypto::{hash_512, KeyPair};
use crate::economics::{
    compute_fee_micro, effective_reputation_staked, emission_reputation_factor, TxPriority,
    APPROX_ALERT_BYTES, APPROX_TX_BYTES, EMISSION_BASE_PER_HOUR_MICRO,
    FEE_SHARE_STORAGE, REPUTATION_DELTA_RELAY, REPUTATION_PUNISH_MIN,
};
use crate::graph::{Alert, Anchor, LocalGraph, Transaction};
use crate::network::{NetworkEvent, P2PNode};
use dashmap::DashMap;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
use tokio::sync::{mpsc, oneshot, RwLock};

/// Команды локального RPC (stats, send, pubkey, recent_txs) и внутренние (эмиссия)
pub enum AdminCommand {
    Stats(oneshot::Sender<serde_json::Value>),
    Pubkey(oneshot::Sender<String>),
    Send {
        to: Vec<u8>,
        amount: u64,
        reply: oneshot::Sender<Result<Transaction, String>>,
    },
    /// Последние транзакции (limit), ответ — JSON массив
    RecentTxs { limit: usize, reply: oneshot::Sender<String> },
    /// Начисление награды за хранение (осаждение)
    EmissionReward(u64),
    /// Заморозить долю репутации (стейкинг); ответ — ok или error
    Stake { fraction: f64, reply: oneshot::Sender<Result<(), String>> },
}

/// Конфигурация узла
pub struct NodeConfig {
    pub initial_balance: u64,
    pub initial_reputation: f64,
    pub enable_chaff: bool,
    pub chaff_probability: f64,
    pub data_dir: String,
    /// Адрес для локального RPC (например "127.0.0.1:9190")
    pub admin_listen: Option<String>,
    /// Интервал осаждения (секунды); 0 = отключено
    pub emission_interval_secs: u64,
}

/// Узел сети «Елена»
pub struct ElenaNode {
    pub keypair: KeyPair,
    pub peer_id: Vec<u8>,
    pub balance: u64,
    pub reputation: Arc<DashMap<Vec<u8>, f64>>,
    pub graph: Arc<RwLock<LocalGraph>>,
    pub p2p: P2PNode,
    pub echo: EchoLocator,
    pub config: NodeConfig,
    admin_rx: Option<mpsc::UnboundedReceiver<AdminCommand>>,
    /// Доля репутации в стейкинге (0.0 .. 0.5)
    staked_fraction: Arc<RwLock<f64>>,
}

impl ElenaNode {
    /// Создаёт узел. keypair: None — сгенерировать новый, Some(kp) — загруженный кошелёк.
    pub async fn new(
        config: NodeConfig,
        keypair: Option<KeyPair>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let keypair = keypair.unwrap_or_else(KeyPair::generate);
        let peer_id = hash_512(keypair.public_key()).to_vec();
        let p2p = P2PNode::new().await?;
        let graph_path = format!("{}/graph.json", config.data_dir.trim_end_matches('/'));
        let graph = Arc::new(RwLock::new(
            LocalGraph::load_from_path(&graph_path).unwrap_or_else(|e| {
                log::warn!("Load graph {}: {}, starting empty", graph_path, e);
                LocalGraph::new()
            }),
        ));
        let reputation = Arc::new(DashMap::new());
        reputation.insert(peer_id.clone(), config.initial_reputation);
        let echo = EchoLocator::new();
        let staked_fraction = Arc::new(RwLock::new(load_stake_fraction(&config.data_dir)));

        let admin_rx = if config.admin_listen.is_some() || config.emission_interval_secs > 0 {
            let (tx, rx) = mpsc::unbounded_channel();
            if let Some(ref addr) = config.admin_listen {
                spawn_admin_listener(addr.clone(), tx.clone());
            }
            if config.emission_interval_secs > 0 {
                spawn_emission_task(
                    config.emission_interval_secs,
                    graph.clone(),
                    reputation.clone(),
                    peer_id.clone(),
                    staked_fraction.clone(),
                    tx,
                );
            }
            Some(rx)
        } else {
            None
        };

        Ok(Self {
            keypair,
            peer_id,
            balance: config.initial_balance,
            reputation,
            graph,
            p2p,
            echo,
            config,
            admin_rx,
            staked_fraction,
        })
    }

    fn compute_anchor(&self) -> Anchor {
        let mut data = Vec::new();
        data.extend_from_slice(&self.balance.to_le_bytes());
        hash_512(&data)
    }

    /// Создаёт платёж и публикует в сеть. Баланс и amount в микро-ELENA; комиссия вычисляется по tokenomics.
    pub async fn create_payment(
        &mut self,
        to: Vec<u8>,
        amount: u64,
    ) -> Result<Transaction, Box<dyn std::error::Error + Send + Sync>> {
        let sender_rep = self.reputation.get(&self.peer_id).map(|r| *r).unwrap_or(0.5);
        let fee = compute_fee_micro(amount, TxPriority::Normal, sender_rep);
        let total = amount.saturating_add(fee);
        if self.balance < total {
            return Err("Insufficient balance (amount + fee)".into());
        }
        let anchor = self.compute_anchor();
        let graph: tokio::sync::RwLockReadGuard<'_, LocalGraph> = self.graph.read().await;
        let parents = graph.recent_tx_ids_for_sender(&self.peer_id, 5);
        drop(graph);

        let tx = Transaction::new(
            self.keypair.public_key().to_vec(),
            to,
            amount,
            anchor,
            parents,
            fee,
            &self.keypair,
        )?;
        self.balance -= total;
        self.p2p.publish_transaction(&tx);
        Ok(tx)
    }

    async fn handle_transaction(
        &mut self,
        tx: Transaction,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if !tx.verify_signature(tx.from.as_slice()) {
            return Err("Invalid signature".into());
        }
        let mut graph: tokio::sync::RwLockWriteGuard<'_, LocalGraph> = self.graph.write().await;
        match graph.add_transaction(tx.clone()) {
            Ok(()) => {
                let graph_path = format!("{}/graph.json", self.config.data_dir.trim_end_matches('/'));
                if let Err(e) = graph.save_to_path(&graph_path) {
                    log::warn!("Save graph: {}", e);
                }
                drop(graph);
                self.update_reputation(&tx.from, REPUTATION_DELTA_RELAY).await;
                if tx.fee > 0 {
                    let storage_share = (tx.fee as f64 * FEE_SHARE_STORAGE) as u64;
                    self.balance = self.balance.saturating_add(storage_share);
                }
                self.p2p.publish_transaction(&tx);
                Ok(())
            }
            Err(e) if e == "Collision detected" => {
                let collision_txs: Vec<Transaction> = graph
                    .find_collisions(&tx.anchor)
                    .into_iter()
                    .cloned()
                    .collect();
                drop(graph);
                if let Some(other) = collision_txs.first() {
                    let alert = Alert {
                        id: hash_512(&tx.anchor[..]),
                        conflicting_tx1: other.id,
                        conflicting_tx2: tx.id,
                        anchor: tx.anchor,
                        discovered_by: self.peer_id.clone(),
                        timestamp: tx.timestamp,
                        propagation_count: 0,
                    };
                    self.p2p.publish_alert(&alert);
                    self.punish_attacker(&tx.from).await;
                }
                Err("Collision detected".into())
            }
                Err(e) => Err::<(), Box<dyn std::error::Error + Send + Sync>>(e.into()),
        }
    }

    async fn update_reputation(&self, node_id: &[u8], delta: f64) {
        let mut entry = self.reputation.entry(node_id.to_vec()).or_insert(0.5);
        *entry = (*entry + delta).min(0.99).max(0.01);
    }

    async fn punish_attacker(&self, node_id: &[u8]) {
        self.reputation.insert(node_id.to_vec(), REPUTATION_PUNISH_MIN);
        log::warn!(
            "Attacker punished: {}",
            hex::encode(&node_id[..node_id.len().min(8)])
        );
    }

    /// Основной цикл: обработка событий из P2P и локального RPC
    pub async fn run(mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut admin_rx = self.admin_rx.take();
        loop {
            if let Some(ref mut rx) = admin_rx {
                tokio::select! {
                    event = self.p2p.recv_event() => {
                        let event = match event {
                            Some(e) => e,
                            None => break,
                        };
                        match event {
                            NetworkEvent::TransactionReceived(tx) => {
                                if let Err(e) = self.handle_transaction(tx).await {
                                    log::error!("handle_transaction: {}", e);
                                }
                            }
                            NetworkEvent::AlertReceived(alert) => {
                                log::info!(
                                    "Alert received anchor: {}",
                                    hex::encode(&alert.anchor[..alert.anchor.len().min(8)])
                                );
                                self.p2p.publish_alert(&alert);
                            }
                            NetworkEvent::PeerConnected(peer) => log::info!("Peer connected: {}", peer),
                            NetworkEvent::PeerDisconnected(peer) => log::info!("Peer disconnected: {}", peer),
                        }
                    }
                    cmd = rx.recv() => {
                        match cmd {
                            Some(AdminCommand::Stats(tx)) => {
                                let v = self.get_stats().await;
                                let _ = tx.send(v);
                            }
                            Some(AdminCommand::Pubkey(tx)) => {
                                let _ = tx.send(hex::encode(self.keypair.public_key()));
                            }
                            Some(AdminCommand::Send { to, amount, reply }) => {
                                let r = self.create_payment(to, amount).await;
                                let _ = reply.send(r.map_err(|e| e.to_string()));
                            }
                            Some(AdminCommand::EmissionReward(amount)) => {
                                self.balance = self.balance.saturating_add(amount);
                                log::debug!("Emission reward +{}", amount);
                            }
                            Some(AdminCommand::Stake { fraction, reply }) => {
                                let r = if fraction >= 0.0 && fraction <= 0.5 {
                                    *self.staked_fraction.write().await = fraction;
                                    let _ = save_stake_fraction(&self.config.data_dir, fraction);
                                    Ok(())
                                } else {
                                    Err("stake fraction must be in [0.0, 0.5]".to_string())
                                };
                                let _ = reply.send(r);
                            }
                            Some(AdminCommand::RecentTxs { limit, reply }) => {
                                let graph = self.graph.read().await;
                                let list = graph.recent_transactions(limit);
                                let json = serde_json::to_string(&list).unwrap_or_else(|_| "[]".into());
                                let _ = reply.send(json);
                            }
                            None => break,
                        }
                    }
                }
            } else {
                match self.p2p.recv_event().await {
                    Some(event) => {
                        match event {
                            NetworkEvent::TransactionReceived(tx) => {
                                if let Err(e) = self.handle_transaction(tx).await {
                                    log::error!("handle_transaction: {}", e);
                                }
                            }
                            NetworkEvent::AlertReceived(alert) => {
                                log::info!(
                                    "Alert received anchor: {}",
                                    hex::encode(&alert.anchor[..alert.anchor.len().min(8)])
                                );
                                self.p2p.publish_alert(&alert);
                            }
                            NetworkEvent::PeerConnected(peer) => log::info!("Peer connected: {}", peer),
                            NetworkEvent::PeerDisconnected(peer) => log::info!("Peer disconnected: {}", peer),
                        }
                    }
                    None => break,
                }
            }
        }
        Ok(())
    }

    pub async fn get_stats(&self) -> serde_json::Value {
        let graph = self.graph.read().await;
        let reputation: HashMap<String, f64> = self
            .reputation
            .iter()
            .map(|e| (hex::encode(e.key()), *e.value()))
            .collect();
        serde_json::json!({
            "peer_id": hex::encode(&self.peer_id),
            "balance": self.balance,
            "reputation": reputation,
            "transactions": graph.transactions.len(),
            "alerts": graph.alerts.len(),
        })
    }
}

fn load_stake_fraction(data_dir: &str) -> f64 {
    let path = format!("{}/stake.json", data_dir.trim_end_matches('/'));
    if let Ok(s) = std::fs::read_to_string(&path) {
        if let Ok(v) = serde_json::from_str::<f64>(&s) {
            return v.clamp(0.0, 0.5);
        }
    }
    0.0
}

fn save_stake_fraction(data_dir: &str, fraction: f64) -> Result<(), std::io::Error> {
    std::fs::create_dir_all(data_dir)?;
    let path = format!("{}/stake.json", data_dir.trim_end_matches('/'));
    std::fs::write(path, serde_json::to_string(&fraction).unwrap_or_else(|_| "0".into()))
}

/// Периодически начисляет награду за хранение (осаждение).
fn spawn_emission_task(
    interval_secs: u64,
    graph: Arc<RwLock<LocalGraph>>,
    reputation: Arc<DashMap<Vec<u8>, f64>>,
    peer_id: Vec<u8>,
    staked_fraction: Arc<RwLock<f64>>,
    admin_tx: mpsc::UnboundedSender<AdminCommand>,
) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(interval_secs));
        interval.tick().await;
        loop {
            interval.tick().await;
            let (storage_bytes, rep, stake) = {
                let g = graph.read().await;
                let bytes = g.transactions.len() * APPROX_TX_BYTES + g.alerts.len() * APPROX_ALERT_BYTES;
                let r = reputation.get(&peer_id).map(|x| *x).unwrap_or(0.5);
                let s = *staked_fraction.read().await;
                (bytes, r, s)
            };
            let rep_eff = effective_reputation_staked(rep, stake);
            let factor = emission_reputation_factor(rep_eff);
            let reward = (EMISSION_BASE_PER_HOUR_MICRO as f64
                * (storage_bytes as f64 / 1_000_000.0)
                * factor
                * (interval_secs as f64 / 3600.0)) as u64;
            if reward > 0 && admin_tx.send(AdminCommand::EmissionReward(reward)).is_err() {
                break;
            }
        }
    });
}

/// Запускает TCP-сервер для локальных команд: "stats" -> JSON, "send <hex_pubkey> <amount>" -> ok/tx_id или error.
fn spawn_admin_listener(addr: String, admin_tx: mpsc::UnboundedSender<AdminCommand>) {
    tokio::spawn(async move {
        let listener = match tokio::net::TcpListener::bind(&addr).await {
            Ok(l) => l,
            Err(e) => {
                log::error!("Admin listen {}: {}", addr, e);
                return;
            }
        };
        log::info!("Admin RPC on {}", addr);
        while let Ok((stream, _)) = listener.accept().await {
            let tx = admin_tx.clone();
            tokio::spawn(async move {
                let (reader, mut writer) = stream.into_split();
                let mut reader = tokio::io::BufReader::new(reader);
                let mut line = String::new();
                if reader.read_line(&mut line).await.is_err() {
                    return;
                }
                let line = line.trim();
                if line.is_empty() {
                    return;
                }
                let parts: Vec<&str> = line.split_ascii_whitespace().collect();
                if parts[0] == "stats" {
                    let (reply_tx, reply_rx) = oneshot::channel();
                    if tx.send(AdminCommand::Stats(reply_tx)).is_err() {
                        return;
                    }
                    match reply_rx.await {
                        Ok(v) => {
                            let out = v.to_string() + "\n";
                            let _ = writer.write_all(out.as_bytes()).await;
                        }
                        Err(_) => {}
                    }
                } else if parts[0] == "pubkey" {
                    let (reply_tx, reply_rx) = oneshot::channel();
                    if tx.send(AdminCommand::Pubkey(reply_tx)).is_err() {
                        return;
                    }
                    match reply_rx.await {
                        Ok(hex_pk) => {
                            let out = hex_pk + "\n";
                            let _ = writer.write_all(out.as_bytes()).await;
                        }
                        Err(_) => {}
                    }
                } else if parts[0] == "send" && parts.len() >= 3 {
                    let to_hex = parts[1];
                    let amount: u64 = match parts[2].parse() {
                        Ok(n) => n,
                        Err(_) => {
                            let _ = writer.write_all(b"error: invalid amount\n").await;
                            return;
                        }
                    };
                    let to = match hex::decode(to_hex) {
                        Ok(b) => b,
                        Err(_) => {
                            let _ = writer.write_all(b"error: invalid pubkey hex\n").await;
                            return;
                        }
                    };
                    let (reply_tx, reply_rx) = oneshot::channel();
                    if tx.send(AdminCommand::Send { to, amount, reply: reply_tx }).is_err() {
                        return;
                    }
                    match reply_rx.await {
                        Ok(Ok(tx)) => {
                            let out = format!("ok {}\n", hex::encode(tx.id));
                            let _ = writer.write_all(out.as_bytes()).await;
                        }
                        Ok(Err(e)) => {
                            let out = format!("error: {}\n", e);
                            let _ = writer.write_all(out.as_bytes()).await;
                        }
                        Err(_) => {}
                    }
                } else if parts[0] == "stake" && parts.len() >= 2 {
                    let fraction: f64 = match parts[1].parse() {
                        Ok(x) => x,
                        Err(_) => {
                            let _ = writer.write_all(b"error: invalid fraction\n").await;
                            return;
                        }
                    };
                    let (reply_tx, reply_rx) = oneshot::channel();
                    if tx.send(AdminCommand::Stake { fraction, reply: reply_tx }).is_err() {
                        return;
                    }
                    match reply_rx.await {
                        Ok(Ok(())) => {
                            let _ = writer.write_all(b"ok\n").await;
                        }
                        Ok(Err(e)) => {
                            let _ = writer.write_all(format!("error: {}\n", e).as_bytes()).await;
                        }
                        Err(_) => {}
                    }
                } else if parts[0] == "recent_txs" {
                    let limit = if parts.len() >= 2 {
                        parts[1].parse().unwrap_or(20)
                    } else {
                        20
                    };
                    let (reply_tx, reply_rx) = oneshot::channel();
                    if tx.send(AdminCommand::RecentTxs { limit, reply: reply_tx }).is_err() {
                        return;
                    }
                    match reply_rx.await {
                        Ok(json) => {
                            let out = json + "\n";
                            let _ = writer.write_all(out.as_bytes()).await;
                        }
                        Err(_) => {}
                    }
                } else if parts[0] == "params" {
                    let out = crate::economics::network_params_json() + "\n";
                    let _ = writer.write_all(out.as_bytes()).await;
                } else {
                    let _ = writer.write_all(b"error: unknown command (use stats, pubkey, recent_txs [N], params, stake <0-0.5>, or send <pubkey_hex> <amount>)\n").await;
                }
            });
        }
    });
}
