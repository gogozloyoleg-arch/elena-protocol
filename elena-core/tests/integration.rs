//! Интеграционный тест: два узла, платёж через инъекцию в заглушку P2P.

use elena_core::node::{ElenaNode, NodeConfig};
use std::sync::Arc;
use std::time::Duration;
use tempfile::tempdir;

#[tokio::test]
async fn two_nodes_payment_via_inject() {
    let _ = env_logger::try_init();
    let dir1 = tempdir().unwrap();
    let dir2 = tempdir().unwrap();

    let config1 = NodeConfig {
        initial_balance: 1000,
        initial_reputation: 0.5,
        enable_chaff: false,
        chaff_probability: 0.0,
        data_dir: dir1.path().to_string_lossy().to_string(),
        admin_listen: None,
        emission_interval_secs: 0,
    };
    let config2 = NodeConfig {
        initial_balance: 1000,
        initial_reputation: 0.5,
        enable_chaff: false,
        chaff_probability: 0.0,
        data_dir: dir2.path().to_string_lossy().to_string(),
        admin_listen: None,
        emission_interval_secs: 0,
    };

    let mut node1 = ElenaNode::new(config1, None).await.unwrap();
    let node2 = ElenaNode::new(config2, None).await.unwrap();

    let tx = node1
        .create_payment(node2.peer_id.clone(), 100)
        .await
        .unwrap();
    assert_eq!(tx.amount, 100);

    let graph2 = Arc::clone(&node2.graph);
    let sender = node2.p2p.event_sender();
    tokio::spawn(async move {
        let _ = node2.run().await;
    });

    let _ = sender.send(elena_core::network::NetworkEvent::TransactionReceived(tx));
    tokio::time::sleep(Duration::from_millis(100)).await;

    let g = graph2.read().await;
    assert!(
        g.transactions.len() >= 1,
        "ожидалась хотя бы одна транзакция в графе узла 2, получено {}",
        g.transactions.len()
    );
}
