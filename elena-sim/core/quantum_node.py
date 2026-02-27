"""
Злой узел с квантовым преимуществом для симуляции атак.
"""

import random
import time
from typing import List, Tuple, Optional, TYPE_CHECKING

from .node import Node
from .transaction import Transaction
from .crypto import compute_anchor, sign_data, tx_content_hash

if TYPE_CHECKING:
    from .graph import NetworkGraph


class QuantumEvilNode(Node):
    """Злоумышленник с имитацией квантового преимущества."""

    def __init__(self, node_id: str, quantum_advantage: float = 0.7):
        super().__init__(node_id, initial_reputation=0.6)
        self.quantum_advantage = quantum_advantage
        self.is_evil = True

    def predict_anchor(self, target_node: Node) -> str:
        """
        Пытается предсказать следующий anchor жертвы (квантовое преимущество).
        С вероятностью advantage * 0.3 угадывает.
        """
        if random.random() > self.quantum_advantage * 0.3:
            return ""  # не угадал
        last_txs = [t.id for t in target_node.my_transactions[-2:]]
        guessed_nonce = random.randint(0, 2**32)
        return compute_anchor(
            target_node.balance,
            last_txs,
            guessed_nonce,
            time.time(),
        )

    def find_weak_peers(self, all_nodes: List[Node], threshold: float = 0.3) -> List[Node]:
        """Находит узлы с низкой репутацией для атаки (квантовый анализ графа)."""
        weak = [n for n in all_nodes if getattr(n, "is_evil", False) is False and n.reputation <= threshold]
        if random.random() < self.quantum_advantage:
            weak = [n for n in all_nodes if getattr(n, "is_evil", False) is False and n.reputation <= threshold + 0.2]
        return weak

    def double_spend_attack(
        self,
        target1: str,
        target2: str,
        amount: float,
    ) -> Tuple[Optional[Transaction], Optional[Transaction]]:
        """
        Пытается провести двойную трату: две транзакции с одинаковым anchor/parents,
        но разными получателями (минимальный временной разрыв).
        """
        tx1 = self.create_transaction(target1, amount)
        if not tx1:
            return (None, None)
        # Вторая транзакция: те же anchor и parents, другой получатель — двойная трата
        nonce2 = tx1.nonce + 1
        timestamp = time.time()
        parent_ids = list(tx1.parents)
        anchor = tx1.anchor
        tx_id2 = tx_content_hash(self.id, target2, amount, nonce2, anchor, parent_ids, timestamp)
        tx2 = Transaction(
            id=tx_id2,
            from_id=self.id,
            to_id=target2,
            amount=amount,
            nonce=nonce2,
            anchor=anchor,
            parents=parent_ids,
            timestamp=timestamp,
            signature=b"",
            is_chaff=False,
        )
        data = tx2.content_for_signature()
        tx2.signature = sign_data(data, self.private_key)
        self.my_transactions.append(tx2)
        self.local_graph[tx_id2] = tx2
        return (tx1, tx2)

    def _split_peers_by_reputation(self, threshold: float = 0.5) -> "Tuple[List[Node], List[Node]]":
        """Делит своих пиров на «сильных» (высокая репутация) и «слабых» (низкая)."""
        strong = [p for p in self.peers if p.reputation >= threshold]
        weak = [p for p in self.peers if p.reputation < threshold]
        if not weak and self.peers:
            weak = self.peers[len(self.peers) // 2 :]
            strong = self.peers[: len(self.peers) // 2]
        return strong, weak

    def sophisticated_double_spend(
        self,
        target1: str,
        target2: str,
        amount: float,
        graph,  # NetworkGraph
        reputation_threshold: float = 0.5,
    ) -> Tuple[Optional[Transaction], Optional[Transaction]]:
        """
        Умная двойная трата: первая транзакция — через «сильных» пиров,
        вторая — с задержкой только через «слабых», чтобы развести потоки по кластерам.
        Гипотеза: если вторая tx долго не доходит до честного кластера, конфликт могут обнаружить позже.
        """
        tx1 = self.create_transaction(target1, amount)
        if not tx1:
            return (None, None)
        strong_peers, weak_peers = self._split_peers_by_reputation(reputation_threshold)
        if not strong_peers:
            strong_peers = list(self.peers)
        if not weak_peers:
            weak_peers = [p for p in self.peers if p not in strong_peers] or list(self.peers)

        # 1. Первую транзакцию — только в «честный» кластер (сильные связи)
        graph.propagate_transaction(tx1, self, first_hop_peers=strong_peers)

        # 2. Вторая транзакция: тот же anchor, другой получатель
        nonce2 = tx1.nonce + 1
        timestamp = time.time()
        parent_ids = list(tx1.parents)
        anchor = tx1.anchor
        tx_id2 = tx_content_hash(self.id, target2, amount, nonce2, anchor, parent_ids, timestamp)
        tx2 = Transaction(
            id=tx_id2,
            from_id=self.id,
            to_id=target2,
            amount=amount,
            nonce=nonce2,
            anchor=anchor,
            parents=parent_ids,
            timestamp=timestamp,
            signature=b"",
            is_chaff=False,
        )
        data = tx2.content_for_signature()
        tx2.signature = sign_data(data, self.private_key)
        self.my_transactions.append(tx2)
        self.local_graph[tx_id2] = tx2

        # 3. Вторую — только через слабые связи (изолированный кластер)
        graph.propagate_transaction(tx2, self, first_hop_peers=weak_peers)

        return (tx1, tx2)
