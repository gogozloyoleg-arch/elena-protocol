//! Реальный P2P через libp2p: TCP + Noise + Yamux, Floodsub для транзакций и алертов.

use crate::graph::{Alert, Transaction};
use libp2p::floodsub::{Floodsub, FloodsubEvent, FloodsubMessage, Topic};
use libp2p::futures::StreamExt;
use libp2p::swarm::{NetworkBehaviour, SwarmEvent};
use libp2p::{identity, noise, tcp, yamux, Multiaddr, PeerId, SwarmBuilder};
use tokio::sync::mpsc;

const TOPIC_TX: &str = "elena-transactions";
const TOPIC_ALERT: &str = "elena-alerts";

#[derive(Clone, Debug)]
pub enum NetworkEvent {
    TransactionReceived(Transaction),
    AlertReceived(Alert),
    PeerConnected(String),
    PeerDisconnected(String),
}

/// Команды для задачи Swarm
enum Command {
    Listen(Multiaddr),
    Dial(Multiaddr),
    PublishTx(Transaction),
    PublishAlert(Alert),
}

/// Поведение сети: только Floodsub (транзакции и алерты)
#[derive(NetworkBehaviour)]
#[behaviour(to_swarm = "FloodsubEvent")]
struct ElenaBehaviour {
    floodsub: Floodsub,
}

/// P2P-узел на libp2p
pub struct P2PNode {
    pub peer_id: String,
    command_tx: mpsc::UnboundedSender<Command>,
    event_tx: mpsc::UnboundedSender<NetworkEvent>,
    event_rx: Option<mpsc::UnboundedReceiver<NetworkEvent>>,
}

impl P2PNode {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let keypair = identity::Keypair::generate_ed25519();
        let peer_id = PeerId::from(keypair.public());

        let mut swarm = SwarmBuilder::with_new_identity()
            .with_tokio()
            .with_tcp(
                tcp::Config::default(),
                noise::Config::new,
                yamux::Config::default,
            )?
            .with_quic()
            .with_behaviour(|key: &libp2p::identity::Keypair| {
                let local_peer_id = key.public().to_peer_id();
                let floodsub = Floodsub::new(local_peer_id);
                ElenaBehaviour { floodsub }
            })?
            .build();

        let topic_tx = Topic::new(TOPIC_TX);
        let topic_alert = Topic::new(TOPIC_ALERT);
        swarm.behaviour_mut().floodsub.subscribe(topic_tx.clone());
        swarm.behaviour_mut().floodsub.subscribe(topic_alert.clone());

        let (command_tx, mut command_rx) = mpsc::unbounded_channel();
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        let event_tx_for_task = event_tx.clone();
        let peer_id_str = peer_id.to_string();

        tokio::spawn(async move {
            let mut swarm = swarm;
            let topic_tx = topic_tx;
            let topic_alert = topic_alert;
            loop {
                tokio::select! {
                    Some(cmd) = command_rx.recv() => {
                        match cmd {
                            Command::Listen(addr) => {
                                if let Err(e) = swarm.listen_on(addr) {
                                    log::warn!("listen_on failed: {}", e);
                                }
                            }
                            Command::Dial(addr) => {
                                if let Err(e) = swarm.dial(addr) {
                                    log::warn!("dial failed: {}", e);
                                }
                            }
                            Command::PublishTx(tx) => {
                                let data = match bincode::serialize(&tx) {
                                    Ok(d) => d,
                                    Err(e) => {
                                        log::warn!("serialize tx: {}", e);
                                        continue;
                                    }
                                };
                                swarm.behaviour_mut().floodsub.publish_any(topic_tx.clone(), data);
                            }
                            Command::PublishAlert(alert) => {
                                let data = match bincode::serialize(&alert) {
                                    Ok(d) => d,
                                    Err(e) => {
                                        log::warn!("serialize alert: {}", e);
                                        continue;
                                    }
                                };
                                swarm.behaviour_mut().floodsub.publish_any(topic_alert.clone(), data);
                            }
                        }
                    }
                    ev = swarm.select_next_some() => {
                        match ev {
                            SwarmEvent::NewListenAddr { address, .. } => {
                                log::info!("Listening on {}", address);
                            }
                            SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                                let _ = event_tx_for_task.send(NetworkEvent::PeerConnected(peer_id.to_string()));
                            }
                            SwarmEvent::ConnectionClosed { peer_id, .. } => {
                                let _ = event_tx_for_task.send(NetworkEvent::PeerDisconnected(peer_id.to_string()));
                            }
                            SwarmEvent::Behaviour(FloodsubEvent::Message(msg)) => {
                                handle_floodsub_message(&msg, &event_tx_for_task, &topic_tx, &topic_alert);
                            }
                            _ => {}
                        }
                    }
                }
            }
        });

        Ok(Self {
            peer_id: peer_id_str,
            command_tx,
            event_tx: event_tx.clone(),
            event_rx: Some(event_rx),
        })
    }

    pub async fn listen_on(&mut self, addr: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let addr: Multiaddr = addr.parse()?;
        self.command_tx.send(Command::Listen(addr))?;
        Ok(())
    }

    pub async fn dial(&mut self, addr: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let addr: Multiaddr = addr.parse()?;
        self.command_tx.send(Command::Dial(addr))?;
        Ok(())
    }

    pub fn publish_transaction(&self, tx: &Transaction) {
        let _ = self.command_tx.send(Command::PublishTx(tx.clone()));
    }

    pub fn publish_alert(&self, alert: &Alert) {
        let _ = self.command_tx.send(Command::PublishAlert(alert.clone()));
    }

    /// Для тестов: вброс события в канал (без отправки в сеть)
    pub fn inject_transaction(&self, tx: Transaction) {
        let _ = self.event_tx.send(NetworkEvent::TransactionReceived(tx));
    }

    pub fn event_sender(&self) -> mpsc::UnboundedSender<NetworkEvent> {
        self.event_tx.clone()
    }

    pub async fn recv_event(&mut self) -> Option<NetworkEvent> {
        match &mut self.event_rx {
            Some(rx) => rx.recv().await,
            None => {
                let (_, mut rx) = mpsc::unbounded_channel();
                rx.recv().await
            }
        }
    }

    pub fn event_receiver(&mut self) -> mpsc::UnboundedReceiver<NetworkEvent> {
        self.event_rx.take().unwrap_or_else(|| {
            let (_, rx) = mpsc::unbounded_channel();
            rx
        })
    }

    pub async fn run(mut self) {
        while self.recv_event().await.is_some() {}
    }
}

fn handle_floodsub_message(
    msg: &FloodsubMessage,
    event_tx: &mpsc::UnboundedSender<NetworkEvent>,
    topic_tx: &Topic,
    topic_alert: &Topic,
) {
    for t in &msg.topics {
        if t == topic_tx {
            if let Ok(tx) = bincode::deserialize::<Transaction>(&msg.data) {
                let _ = event_tx.send(NetworkEvent::TransactionReceived(tx));
            }
            break;
        }
        if t == topic_alert {
            if let Ok(alert) = bincode::deserialize::<Alert>(&msg.data) {
                let _ = event_tx.send(NetworkEvent::AlertReceived(alert));
            }
            break;
        }
    }
}
