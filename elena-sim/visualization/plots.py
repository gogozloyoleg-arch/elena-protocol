"""
Построение графиков метрик и состояния сети.
"""

from pathlib import Path
from typing import Optional, List, Dict, Any

import matplotlib.pyplot as plt
import numpy as np


def plot_metrics(
    metrics: "MetricsCollector",
    save_path: Optional[Path] = None,
) -> None:
    """Строит графики по метрикам: throughput, репутация, обнаружение."""
    fig, axes = plt.subplots(2, 2, figsize=(12, 10))
    # Пропускная способность
    if metrics.tx_throughput:
        axes[0, 0].plot(metrics.tx_throughput, alpha=0.7)
        axes[0, 0].set_title("Пропускная способность (сообщений/шаг)")
        axes[0, 0].set_xlabel("Шаг")
    # Репутация (средняя по сети за шаги)
    if metrics.reputation_history:
        steps = [h["step"] for h in metrics.reputation_history]
        avg_rep = [
            np.mean(list(h["reputations"].values())) if h["reputations"] else 0
            for h in metrics.reputation_history
        ]
        axes[0, 1].plot(steps, avg_rep)
        axes[0, 1].set_title("Средняя репутация по сети")
        axes[0, 1].set_xlabel("Шаг")
    # Время обнаружения (гистограмма)
    if metrics.detection_times:
        axes[1, 0].hist(metrics.detection_times, bins=min(20, len(metrics.detection_times)) or 1, edgecolor="black")
        axes[1, 0].set_title("Распределение времени обнаружения (шаги)")
    # Сводка текстом
    summary = metrics.get_summary()
    text = "\n".join(f"{k}: {v}" for k, v in summary.items())
    axes[1, 1].text(0.1, 0.5, text, fontsize=10, verticalalignment="center", family="monospace")
    axes[1, 1].axis("off")
    axes[1, 1].set_title("Сводка метрик")
    plt.tight_layout()
    if save_path:
        plt.savefig(save_path, dpi=150)
    plt.close()


def plot_network_state(
    nodes: Dict[str, Any],
    transactions: Dict[str, Any],
    save_path: Optional[Path] = None,
) -> None:
    """Визуализирует состояние сети: узлы и количество транзакций."""
    import networkx as nx
    G = nx.Graph()
    for nid, node in nodes.items():
        G.add_node(nid, reputation=getattr(node, "reputation", 0.5))
    for nid, node in nodes.items():
        for peer in getattr(node, "peers", []):
            if hasattr(peer, "id"):
                G.add_edge(nid, peer.id)
    pos = nx.spring_layout(G, seed=42, k=0.5)
    fig, ax = plt.subplots(figsize=(10, 8))
    reputations = [G.nodes[n].get("reputation", 0.5) for n in G.nodes()]
    nx.draw(G, pos, node_color=reputations, cmap="RdYlGn", node_size=50, with_labels=False, ax=ax)
    plt.colorbar(plt.cm.ScalarMappable(cmap="RdYlGn", norm=plt.Normalize(0, 1)), ax=ax, label="Репутация")
    ax.set_title("Граф сети (цвет = репутация)")
    if save_path:
        plt.savefig(save_path, dpi=150)
    plt.close()


def plot_reputation_history(
    reputation_history: List[Dict],
    save_path: Optional[Path] = None,
) -> None:
    """График истории средней репутации по шагам."""
    if not reputation_history:
        return
    steps = [h["step"] for h in reputation_history]
    avg = [np.mean(list(h["reputations"].values())) if h["reputations"] else 0 for h in reputation_history]
    plt.figure(figsize=(8, 5))
    plt.plot(steps, avg)
    plt.xlabel("Шаг")
    plt.ylabel("Средняя репутация")
    plt.title("История репутации сети")
    if save_path:
        plt.savefig(save_path, dpi=150)
    plt.close()
