"""
Сбор метрик симуляции: обнаружение конфликтов, атаки, репутация, пропускная способность.
"""

import random
from typing import List, Dict, Any, Optional

import numpy as np


class MetricsCollector:
    """Собирает и агрегирует метрики симуляции."""

    def __init__(self):
        self.detection_times: List[float] = []
        self.false_positives: int = 0
        self.successful_attacks: int = 0
        self.propagation_speed: List[float] = []
        self.reputation_history: List[dict] = []
        self.tx_throughput: List[int] = []
        self.alerts_created: int = 0
        self.conflicts_detected: int = 0
        self.nodes_received_alert: List[int] = []
        # Расширенные метрики (METRICS_TO_COLLECT)
        self.avg_reputation: List[float] = []
        self.reputation_distribution: List[list] = []
        self.tx_confidence_5: List[float] = []
        self.tx_confidence_10: List[float] = []
        self.tx_confidence_20: List[float] = []
        self.alert_propagation_time: List[float] = []
        self.false_positive_rate: float = 0.0
        self.network_diameter: int = 0
        self.avg_path_length: float = 0.0

    def record_detection(self, detection_time: float) -> None:
        """Фиксирует время обнаружения конфликта (в шагах)."""
        self.detection_times.append(detection_time)
        self.conflicts_detected += 1

    def record_attack_result(self, success: bool) -> None:
        """Фиксирует результат атаки."""
        if success:
            self.successful_attacks += 1

    def record_false_positive(self) -> None:
        """Фиксирует ложное срабатывание."""
        self.false_positives += 1

    def record_propagation_speed(self, steps: float) -> None:
        """Фиксирует скорость распространения алерта."""
        self.propagation_speed.append(steps)

    def record_throughput(self, count: int) -> None:
        """Фиксирует пропускную способность за шаг."""
        self.tx_throughput.append(count)

    def record_reputation_snapshot(self, step: int, reputations: dict) -> None:
        """Сохраняет снимок репутаций узлов на шаге."""
        self.reputation_history.append({"step": step, "reputations": dict(reputations)})

    def calculate_average_confidence(self, graph, sample_txs: int = 100, sample_nodes: int = 10) -> Dict[str, float]:
        """Средняя уверенность в транзакциях (последние sample_txs, опрос sample_nodes узлов)."""
        if not graph.transactions or not graph.nodes:
            return {"mean": 0.0, "median": 0.0, "std": 0.0}
        nodes_list = list(graph.nodes.values())
        txs = list(graph.transactions.values())[-sample_txs:]
        k_nodes = min(sample_nodes, len(nodes_list))
        confidences = []
        for tx in txs:
            sample = random.sample(nodes_list, k_nodes)
            confs = [n.get_confidence(tx.id) for n in sample]
            confidences.append(np.mean(confs))
        if not confidences:
            return {"mean": 0.0, "median": 0.0, "std": 0.0}
        return {
            "mean": float(np.mean(confidences)),
            "median": float(np.median(confidences)),
            "std": float(np.std(confidences)),
        }

    def get_summary(self) -> dict:
        """Возвращает сводку метрик."""
        avg_detection = sum(self.detection_times) / len(self.detection_times) if self.detection_times else 0
        avg_propagation = sum(self.propagation_speed) / len(self.propagation_speed) if self.propagation_speed else 0
        peak_throughput = max(self.tx_throughput) if self.tx_throughput else 0
        last_rep = self.reputation_history[-1]["reputations"] if self.reputation_history else {}
        avg_rep = sum(last_rep.values()) / len(last_rep) if last_rep else 0
        return {
            "detection_times_count": len(self.detection_times),
            "avg_detection_time_steps": avg_detection,
            "false_positives": self.false_positives,
            "successful_attacks": self.successful_attacks,
            "conflicts_detected": self.conflicts_detected,
            "avg_propagation_speed": avg_propagation,
            "peak_throughput": peak_throughput,
            "reputation_snapshots": len(self.reputation_history),
            "alerts_created": self.alerts_created,
            "avg_reputation": avg_rep,
            "avg_reputation_history": self.avg_reputation,
            "false_positive_rate": self.false_positive_rate,
            "network_diameter": getattr(self, "network_diameter", 0),
            "avg_path_length": getattr(self, "avg_path_length", 0.0),
        }
