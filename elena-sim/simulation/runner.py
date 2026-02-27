"""
Запуск симуляции: инициализация графа, шаги, сбор метрик.
"""

import random
from typing import List, Optional

from core import Node, QuantumEvilNode, NetworkGraph
from config import SIMULATION_PARAMS, REPUTATION_PARAMS
from .metrics import MetricsCollector


class SimulationRunner:
    """Запускает симуляцию: создаёт граф, связывает узлы, выполняет шаги."""

    def __init__(
        self,
        num_nodes: int = None,
        num_evil: int = None,
        quantum_advantage: float = None,
        rewiring_interval: int = None,
        rewiring_prob: float = None,
        chaff_prob: float = None,
        tx_per_step: int = None,
    ):
        params = SIMULATION_PARAMS
        self.num_nodes = num_nodes or params["num_nodes"]
        self.num_evil = num_evil or params["num_evil_nodes"]
        self.quantum_advantage = quantum_advantage or params["quantum_advantage"]
        self.rewiring_interval = rewiring_interval if rewiring_interval is not None else params["rewiring_interval"]
        self.rewiring_prob = rewiring_prob if rewiring_prob is not None else 0.1
        self.chaff_prob = chaff_prob if chaff_prob is not None else params["chaff_probability"]
        self.tx_per_step = tx_per_step or params["tx_per_step"]

        self.graph = NetworkGraph()
        self.metrics = MetricsCollector()
        self.evil_nodes: List[QuantumEvilNode] = []
        self.honest_nodes: List[Node] = []

    def build_network(self) -> None:
        """Создаёт узлы и рёбра графа."""
        initial_rep = REPUTATION_PARAMS.get("initial_reputation", 0.5)
        # Честные узлы
        for i in range(self.num_nodes - self.num_evil):
            node = Node(node_id=f"node_{i}", initial_reputation=initial_rep)
            self.graph.add_node(node)
            self.honest_nodes.append(node)
        # Злые узлы (начинают с чуть выше репутации)
        for i in range(self.num_evil):
            evil = QuantumEvilNode(
                node_id=f"evil_{i}" if self.num_evil > 1 else "evil_0",
                quantum_advantage=self.quantum_advantage,
            )
            evil.reputation = min(initial_rep + 0.01, 0.6)
            self.graph.add_node(evil)
            self.evil_nodes.append(evil)
        # Связи: каждый узел с peer_degree_min..peer_degree_max соседями
        all_nodes = list(self.graph.nodes.values())
        degree_min = SIMULATION_PARAMS.get("peer_degree_min", 3)
        degree_max = SIMULATION_PARAMS.get("peer_degree_max", 10)
        for i, node in enumerate(all_nodes):
            degree = random.randint(degree_min, min(degree_max, len(all_nodes) - 1))
            candidates = [n for j, n in enumerate(all_nodes) if j != i and n not in node.peers]
            random.shuffle(candidates)
            for k in range(min(degree, len(candidates))):
                self.graph.add_edge(node.id, candidates[k].id)
        self._record_network_metrics()

    def step(self, step_id: int) -> int:
        """
        Один шаг симуляции: случайные транзакции, опционально chaff и rewiring.
        Возвращает число обработанных сообщений (throughput).
        """
        messages_this_step = 0
        node_list = list(self.graph.nodes.values())
        for _ in range(self.tx_per_step):
            sender = random.choice(node_list)
            others = [n for n in node_list if n.id != sender.id]
            if not others:
                continue
            receiver = random.choice(others)
            amount = round(random.uniform(1.0, 50.0), 2)
            tx = sender.create_transaction(receiver.id, amount)
            if tx:
                self.graph.propagate_transaction(tx, sender)
                messages_this_step += len(sender.peers) + 1
        if self.chaff_prob > 0 and random.random() < self.chaff_prob * self.num_nodes:
            self.graph.generate_chaff(self.chaff_prob)
            messages_this_step += 10
        if self.rewiring_interval > 0 and step_id > 0 and step_id % self.rewiring_interval == 0:
            self.graph.rewire_peers(self.rewiring_prob)
        # Естественное затухание репутации каждый шаг
        for node in self.graph.nodes.values():
            node.step_decay()
        reputations = {nid: n.reputation for nid, n in self.graph.nodes.items()}
        self.metrics.record_reputation_snapshot(step_id, reputations)
        rep_values = list(reputations.values())
        if rep_values:
            self.metrics.avg_reputation.append(sum(rep_values) / len(rep_values))
            self.metrics.reputation_distribution.append(rep_values)
        self.metrics.record_throughput(messages_this_step)
        return messages_this_step

    def _record_network_metrics(self) -> None:
        """Записывает диаметр и среднюю длину пути графа (после build_network)."""
        try:
            import networkx as nx
            G = self.graph._nx_graph
            if G.number_of_nodes() < 2:
                return
            if not nx.is_connected(G):
                self.metrics.network_diameter = -1
                self.metrics.avg_path_length = -1.0
                return
            self.metrics.network_diameter = nx.diameter(G)
            self.metrics.avg_path_length = nx.average_shortest_path_length(G)
        except Exception:
            pass
