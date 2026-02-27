"""
Классы транзакций и алертов для сети Елена.
"""

from dataclasses import dataclass, field
from typing import List


@dataclass
class Transaction:
    """Транзакция в сети Елена."""

    id: str  # хеш от содержимого
    from_id: str  # ID отправителя
    to_id: str  # ID получателя
    amount: float
    nonce: int  # случайное число
    anchor: str  # SHA3-512( balance | last_tx1 | last_tx2 | nonce | timestamp )
    parents: List[str]  # до 5 предыдущих транзакций
    timestamp: float
    signature: bytes  # имитация Dilithium-подписи
    is_chaff: bool = False  # шумовая транзакция?

    def content_for_signature(self) -> str:
        """Данные, подписываемые отправителем."""
        parents_str = "|".join(self.parents[:5])
        return f"{self.from_id}|{self.to_id}|{self.amount}|{self.nonce}|{self.anchor}|{parents_str}|{self.timestamp}"


@dataclass
class Alert:
    """Сигнал тревоги о конфликте (двойная трата)."""

    id: str
    conflicting_tx1: str
    conflicting_tx2: str
    anchor: str
    discovered_by: str
    propagation_count: int = 0
