"""
Управление графом сети Елена: узлы, рёбра, распространение транзакций и алертов.
"""

import random
from typing import List, Optional

import networkx as nx

from .node import Node
from .transaction import Transaction, Alert
from .quantum_node import QuantumEvilNode

try:
    from config import REPUTATION_PARAMS
except ImportError:
    REPUTATION_PARAMS = {"min_reputation": 0.01, "penalty_double_spend": 0.2}


class NetworkGraph:
    """Граф сети: узлы, транзакции, алерты и распространение."""

    def __init__(self):
        self.nodes: dict[str, Node] = {}  # node_id -> Node
        self.transactions: dict[str, Transaction] = {}  # tx_id -> Transaction
        self.alerts: dict[str, Alert] = {}  # alert_id -> Alert
        self._nx_graph = nx.Graph()  # для топологии и rewiring

    def add_node(self, node: Node) -> None:
        """Добавляет узел в сеть."""
        self.nodes[node.id] = node
        self._nx_graph.add_node(node.id)
        node.set_network(self)

    def add_edge(self, node1_id: str, node2_id: str) -> None:
        """Создаёт связь между узлами (пиры)."""
        if node1_id not in self.nodes or node2_id not in self.nodes:
            return
        n1, n2 = self.nodes[node1_id], self.nodes[node2_id]
        if n2 not in n1.peers:
            n1.peers.append(n2)
        if n1 not in n2.peers:
            n2.peers.append(n1)
        self._nx_graph.add_edge(node1_id, node2_id)

    def propagate_transaction(
        self,
        tx: Transaction,
        start_node: Node,
        first_hop_peers: Optional[List[Node]] = None,
    ) -> None:
        """
        Распространяет транзакцию по сети от start_node.
        Если first_hop_peers задан — только эти пиры получают tx на первом шаге (остальная сеть — через них).
        """
        self.transactions[tx.id] = tx
        visited = {start_node.id}
        initial = first_hop_peers if first_hop_peers is not None else list(start_node.peers)
        stack: List[Node] = [p for p in initial if p.id not in visited]
        while stack:
            node = stack.pop()
            if node.id in visited:
                continue
            visited.add(node.id)
            if node.receive_transaction(tx):
                for peer in node.peers:
                    if peer.id not in visited:
                        stack.append(peer)

    def propagate_alert(self, alert: Alert, start_node: Node) -> None:
        """Распространяет алерт с высоким приоритетом по сети и снижает репутацию виновного."""
        self.alerts[alert.id] = alert
        # Снижаем репутацию отправителя конфликтующих транзакций
        for tx_id in (alert.conflicting_tx1, alert.conflicting_tx2):
            if tx_id in self.transactions:
                sender_id = self.transactions[tx_id].from_id
                if sender_id in self.nodes:
                    penalty = REPUTATION_PARAMS.get("penalty_double_spend", 0.2)
                    min_rep = REPUTATION_PARAMS.get("min_reputation", 0.01)
                    self.nodes[sender_id].reputation = max(min_rep, self.nodes[sender_id].reputation - penalty)
        visited = {start_node.id}
        stack: List[Node] = list(start_node.peers)
        while stack:
            node = stack.pop()
            if node.id in visited:
                continue
            visited.add(node.id)
            node.receive_alert(Alert(
                id=alert.id,
                conflicting_tx1=alert.conflicting_tx1,
                conflicting_tx2=alert.conflicting_tx2,
                anchor=alert.anchor,
                discovered_by=alert.discovered_by,
                propagation_count=alert.propagation_count + 1,
            ))
            for peer in node.peers:
                if peer.id not in visited:
                    stack.append(peer)

    def rewire_peers(self, rewiring_prob: float = 0.1) -> None:
        """Динамически меняет топологию (защита от квантового анализа)."""
        if len(self.nodes) < 3:
            return
        node_ids = list(self.nodes.keys())
        for nid in node_ids:
            node = self.nodes[nid]
            if not node.peers or random.random() > rewiring_prob:
                continue
            # Удаляем одно случайное ребро и добавляем новое к случайному узлу
            peer = random.choice(node.peers)
            node.peers.remove(peer)
            peer.peers.remove(node)
            self._nx_graph.remove_edge(nid, peer.id)
            other = random.choice([x for x in node_ids if x != nid and self.nodes[x] not in node.peers])
            self.add_edge(nid, other)

    def generate_chaff(self, prob: float = 0.05) -> None:
        """Генерирует шумовые транзакции (chaff) от случайных узлов."""
        for node in self.nodes.values():
            if getattr(node, "is_evil", False):
                continue
            if random.random() > prob:
                continue
            targets = [x.id for x in self.nodes.values() if x.id != node.id]
            if not targets:
                continue
            to_id = random.choice(targets)
            tx = node.create_transaction(to_id, 0.01)
            if tx:
                tx.is_chaff = True
                self.propagate_transaction(tx, node)
