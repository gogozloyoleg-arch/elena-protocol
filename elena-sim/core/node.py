"""
Базовый класс узла сети Елена.
"""

import random
import time
from typing import List, Optional, TYPE_CHECKING

from .crypto import compute_anchor, sign_data, verify_signature, generate_keypair, tx_content_hash
from .transaction import Transaction, Alert

try:
    from config import REPUTATION_PARAMS
except ImportError:
    REPUTATION_PARAMS = {
        "reward_per_tx_forwarded": 0.001,
        "reward_per_alert_propagated": 0.01,
        "decay_per_step": 0.0001,
        "max_reputation": 0.99,
        "min_reputation": 0.01,
    }

if TYPE_CHECKING:
    from .graph import NetworkGraph


class Node:
    """Узел сети с локальным графом транзакций и репутацией."""

    def __init__(self, node_id: str, initial_reputation: float = 0.5):
        self.id = node_id
        self.reputation = initial_reputation
        self.balance = 1000.0  # начальный баланс
        self.public_key, self.private_key = generate_keypair()

        # Локальный граф (транзакции, которые знает узел)
        self.local_graph: dict[str, Transaction] = {}  # tx_id -> Transaction
        self.known_balances: dict[str, float] = {node_id: 1000.0}  # node_id -> balance (локальное мнение)
        self.peers: List["Node"] = []  # связи с другими узлами
        self.pending_alerts: dict[str, Alert] = {}  # полученные алерты

        # История
        self.my_transactions: List[Transaction] = []  # исходящие транзакции
        self.received_alerts: List[Alert] = []  # полученные алерты

        # Конфликтующие транзакции (по алертам)
        self.conflicting_tx_ids: set = set()

        # Ссылка на граф для распространения (устанавливается извне)
        self._network: Optional["NetworkGraph"] = None

    def set_network(self, network: "NetworkGraph") -> None:
        """Устанавливает ссылку на граф сети для распространения."""
        self._network = network

    def compute_anchor(self) -> str:
        """Вычисляет текущий якорь на основе своего состояния."""
        last_tx_ids = [t.id for t in self.my_transactions[-2:]] if self.my_transactions else []
        return compute_anchor(
            self.balance,
            last_tx_ids,
            random.randint(0, 2**32),
            time.time(),
        )

    def create_transaction(self, to_node: str, amount: float) -> Optional[Transaction]:
        """Создает новую транзакцию."""
        if amount <= 0 or amount > self.balance:
            return None
        nonce = random.randint(0, 2**32)
        timestamp = time.time()
        last_tx_ids = [t.id for t in self.my_transactions[-2:]]
        anchor = compute_anchor(self.balance, last_tx_ids, nonce, timestamp)
        parent_ids = [t.id for t in self.my_transactions[-5:]]

        tx_id = tx_content_hash(self.id, to_node, amount, nonce, anchor, parent_ids, timestamp)
        tx = Transaction(
            id=tx_id,
            from_id=self.id,
            to_id=to_node,
            amount=amount,
            nonce=nonce,
            anchor=anchor,
            parents=parent_ids,
            timestamp=timestamp,
            signature=b"",
            is_chaff=False,
        )
        data = tx.content_for_signature()
        tx.signature = sign_data(data, self.private_key)

        self.balance -= amount
        self.my_transactions.append(tx)
        self.local_graph[tx_id] = tx
        self.known_balances[self.id] = self.balance
        return tx

    def receive_transaction(self, tx: Transaction) -> bool:
        """
        Обрабатывает входящую транзакцию.
        Проверка подписи, коллизий; добавление в граф; распространение.
        """
        if tx.id in self.local_graph:
            return True  # уже знаем

        data = tx.content_for_signature()
        if not self._network or tx.from_id not in self._network.nodes:
            return False
        sender_public_key = self._network.nodes[tx.from_id].public_key
        if not verify_signature(data, tx.signature, sender_public_key):
            return False

        # Проверка коллизий: две транзакции от одного отправителя с одним anchor или конфликт по балансу
        for existing_id, existing_tx in self.local_graph.items():
            if existing_tx.from_id != tx.from_id:
                continue
            if existing_tx.id == tx.id:
                continue
            # Одна и та же "история" (anchor/parents) — подозрение на двойную трату
            if existing_tx.anchor == tx.anchor and existing_tx.amount == tx.amount:
                alert = Alert(
                    id=f"alert_{tx.id}_{existing_id}",
                    conflicting_tx1=tx.id,
                    conflicting_tx2=existing_id,
                    anchor=tx.anchor,
                    discovered_by=self.id,
                    propagation_count=0,
                )
                self.receive_alert(alert)
                if self._network:
                    self._network.propagate_alert(alert, self)
                return False
            # Разные получатели при том же anchor — конфликт
            if existing_tx.anchor == tx.anchor and existing_tx.to_id != tx.to_id:
                alert = Alert(
                    id=f"alert_{tx.id}_{existing_id}",
                    conflicting_tx1=tx.id,
                    conflicting_tx2=existing_id,
                    anchor=tx.anchor,
                    discovered_by=self.id,
                    propagation_count=0,
                )
                self.receive_alert(alert)
                if self._network:
                    self._network.propagate_alert(alert, self)
                return False

        self.local_graph[tx.id] = tx
        self.conflicting_tx_ids.discard(tx.id)

        # Обновляем локальный баланс отправителя/получателя
        if tx.from_id in self.known_balances:
            self.known_balances[tx.from_id] = self.known_balances.get(tx.from_id, 1000.0) - tx.amount
        else:
            self.known_balances[tx.from_id] = 1000.0 - tx.amount
        self.known_balances[tx.to_id] = self.known_balances.get(tx.to_id, 1000.0) + tx.amount

        # Награда за пересылку транзакции (узел принял и распространяет)
        rp = REPUTATION_PARAMS
        self.reputation = min(
            self.reputation + rp.get("reward_per_tx_forwarded", 0.001),
            rp.get("max_reputation", 0.99),
        )

        if self._network:
            self._network.propagate_transaction(tx, self)
        return True

    def receive_alert(self, alert: Alert) -> None:
        """Обрабатывает сигнал тревоги: помечает конфликт, награда за распространение, распространяет."""
        if alert.id in self.pending_alerts:
            return
        self.pending_alerts[alert.id] = alert
        self.received_alerts.append(alert)
        self.conflicting_tx_ids.add(alert.conflicting_tx1)
        self.conflicting_tx_ids.add(alert.conflicting_tx2)

        for tx_id in (alert.conflicting_tx1, alert.conflicting_tx2):
            if tx_id in self.local_graph:
                sender = self.local_graph[tx_id].from_id
                self.known_balances[sender] = self.known_balances.get(sender, 1000.0)
        # Награда за распространение алерта (не за открытие, а за пересылку)
        if alert.discovered_by != self.id:
            rp = REPUTATION_PARAMS
            self.reputation = min(
                self.reputation + rp.get("reward_per_alert_propagated", 0.01),
                rp.get("max_reputation", 0.99),
            )
        if self._network and alert.discovered_by != self.id:
            self._network.propagate_alert(alert, self)

    def get_confidence(self, tx_id: str) -> float:
        """
        Вычисляет уверенность в транзакции (0-1).
        Чем больше ссылок от узлов с высокой репутацией, тем выше confidence.
        """
        if tx_id not in self.local_graph:
            return 0.0
        if tx_id in self.conflicting_tx_ids:
            return 0.0
        tx = self.local_graph[tx_id]
        score = 0.5
        for other_tx in self.local_graph.values():
            if tx_id in other_tx.parents:
                # Узел, создавший other_tx, "подтвердил" нашу транзакцию
                creator_id = other_tx.from_id
                creator_rep = self._network.nodes[creator_id].reputation if self._network and creator_id in self._network.nodes else 0.5
                score += 0.1 * creator_rep
        return min(1.0, score)

    def step_decay(self) -> None:
        """Естественное затухание репутации (чтобы неактивные узлы теряли вес)."""
        rp = REPUTATION_PARAMS
        self.reputation = max(
            self.reputation - rp.get("decay_per_step", 0.0001),
            rp.get("min_reputation", 0.01),
        )
