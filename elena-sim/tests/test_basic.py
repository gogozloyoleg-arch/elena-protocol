"""
Базовые тесты симулятора сети Елена.
"""

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from core import Node, QuantumEvilNode, NetworkGraph, Transaction, Alert, compute_anchor, generate_keypair, sign_data, verify_signature
from core.crypto import tx_content_hash


def test_anchor_computation():
    anchor = compute_anchor(1000.0, [], 12345, 1.0)
    assert isinstance(anchor, str) and len(anchor) == 128  # sha512 hex


def test_keypair_and_signature():
    pub, priv = generate_keypair()
    data = "test_data"
    sig = sign_data(data, priv)
    assert verify_signature(data, sig, pub)
    assert not verify_signature("other", sig, pub)


def test_node_creation():
    n = Node("alice", initial_reputation=0.6)
    assert n.id == "alice"
    assert n.reputation == 0.6
    assert n.balance == 1000.0
    assert len(n.peers) == 0


def test_node_create_transaction():
    g = NetworkGraph()
    a = Node("a")
    b = Node("b")
    g.add_node(a)
    g.add_node(b)
    g.add_edge("a", "b")
    tx = a.create_transaction("b", 10.0)
    assert tx is not None
    assert tx.from_id == "a" and tx.to_id == "b" and tx.amount == 10.0
    assert a.balance == 990.0


def test_quantum_evil_double_spend():
    g = NetworkGraph()
    evil = QuantumEvilNode("evil", quantum_advantage=0.7)
    a = Node("a")
    b = Node("b")
    g.add_node(evil)
    g.add_node(a)
    g.add_node(b)
    g.add_edge("evil", "a")
    g.add_edge("evil", "b")
    g.add_edge("a", "b")
    tx1, tx2 = evil.double_spend_attack("a", "b", 100.0)
    assert tx1 is not None and tx2 is not None
    assert tx1.anchor == tx2.anchor
    assert tx1.to_id != tx2.to_id


def test_conflict_detection():
    g = NetworkGraph()
    evil = QuantumEvilNode("evil", quantum_advantage=0.0)
    a = Node("a")
    b = Node("b")
    g.add_node(evil)
    g.add_node(a)
    g.add_node(b)
    g.add_edge("evil", "a")
    g.add_edge("evil", "b")
    g.add_edge("a", "b")
    tx1, tx2 = evil.double_spend_attack("a", "b", 50.0)
    g.propagate_transaction(tx1, evil)
    g.propagate_transaction(tx2, evil)
    # Хотя бы один узел должен обнаружить конфликт (a или b имеют обе tx)
    has_alert = any(tx1.id in n.conflicting_tx_ids or tx2.id in n.conflicting_tx_ids for n in g.nodes.values())
    assert has_alert or len(g.alerts) >= 1


def test_simulation_run():
    from simulation.runner import SimulationRunner
    runner = SimulationRunner(num_nodes=20, num_evil=0, tx_per_step=3)
    runner.build_network()
    assert len(runner.graph.nodes) == 20
    for step in range(10):
        runner.step(step)
    assert len(runner.metrics.tx_throughput) == 10


if __name__ == "__main__":
    test_anchor_computation()
    test_keypair_and_signature()
    test_node_creation()
    test_node_create_transaction()
    test_quantum_evil_double_spend()
    test_conflict_detection()
    test_simulation_run()
    print("All tests passed.")
