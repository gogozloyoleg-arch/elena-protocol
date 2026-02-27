#!/usr/bin/env python3
"""
Построение графиков по результатам A/B тестов сети «Елена».
Запуск: python3 plot_results.py results/ab_tests_YYYYMMDD_HHMMSS/results.csv
"""

import pandas as pd
import matplotlib.pyplot as plt
import numpy as np
import sys
import os


def plot_results(csv_file: str) -> None:
    df = pd.read_csv(csv_file)
    if "test_id" not in df.columns:
        print("Ожидаются колонки: test_id, chaff, rewiring, detection_time, alert_coverage, ...")
        return

    df["test_label"] = df["test_id"]
    n = len(df)
    x = np.arange(n)
    width = 0.35

    try:
        plt.style.use("seaborn-v0_8-whitegrid")
    except Exception:
        pass

    fig, axes = plt.subplots(2, 3, figsize=(14, 9))
    fig.suptitle('A/B тесты сети «Елена»', fontsize=14)

    # 1. Время обнаружения
    ax = axes[0, 0]
    bars = ax.bar(x, df["detection_time"], color="skyblue", edgecolor="navy", alpha=0.8)
    ax.set_title("Время обнаружения конфликта (шаги)")
    ax.set_ylabel("шаги")
    ax.set_xticks(x)
    ax.set_xticklabels(df["test_label"], rotation=45, ha="right")
    ax.set_ylim(0, max(df["detection_time"].max(), 1) + 1)
    for bar, val in zip(bars, df["detection_time"]):
        ax.text(bar.get_x() + bar.get_width() / 2, bar.get_height() + 0.05, str(val), ha="center", va="bottom", fontsize=8)

    # 2. Покрытие Alert
    ax = axes[0, 1]
    bars = ax.bar(x, df["alert_coverage"], color="lightgreen", edgecolor="green", alpha=0.8)
    ax.set_title("Узлы, получившие Alert (%)")
    ax.set_ylabel("%")
    ax.set_xticks(x)
    ax.set_xticklabels(df["test_label"], rotation=45, ha="right")
    ax.set_ylim(0, 105)
    for bar, val in zip(bars, df["alert_coverage"]):
        ax.text(bar.get_x() + bar.get_width() / 2, bar.get_height() + 1, f"{val}", ha="center", va="bottom", fontsize=8)

    # 3. Пиковая нагрузка
    ax = axes[0, 2]
    bars = ax.bar(x, df["peak_load"], color="salmon", edgecolor="darkred", alpha=0.8)
    ax.set_title("Пиковая нагрузка (сообщ/шаг)")
    ax.set_ylabel("сообщений/шаг")
    ax.set_xticks(x)
    ax.set_xticklabels(df["test_label"], rotation=45, ha="right")
    for bar, val in zip(bars, df["peak_load"]):
        ax.text(bar.get_x() + bar.get_width() / 2, bar.get_height() + 2, str(int(val)), ha="center", va="bottom", fontsize=8)

    # 4. Репутация злого узла до/после
    ax = axes[1, 0]
    if "evil_reputation_before" in df.columns and "evil_reputation_after" in df.columns:
        ax.bar(x - width / 2, df["evil_reputation_before"], width, label="До атаки", color="orange", alpha=0.8)
        ax.bar(x + width / 2, df["evil_reputation_after"], width, label="После атаки", color="red", alpha=0.8)
    ax.set_title("Репутация злого узла")
    ax.set_ylabel("репутация")
    ax.set_xticks(x)
    ax.set_xticklabels(df["test_label"], rotation=45, ha="right")
    ax.legend()
    ax.set_ylim(0, 1.1)

    # 5. Успешные атаки и ложные срабатывания
    ax = axes[1, 1]
    ax2 = ax.twinx()
    if "successful_attack" in df.columns:
        b1 = ax.bar(x - width / 2, df["successful_attack"], width, label="Успешных атак", color="darkred", alpha=0.7)
    if "false_positives" in df.columns:
        b2 = ax2.bar(x + width / 2, df["false_positives"], width, label="Ложных срабатываний", color="purple", alpha=0.5)
    ax.set_ylabel("успешных атак", color="darkred")
    ax2.set_ylabel("ложных срабатываний", color="purple")
    ax.set_title("Атаки и ошибки")
    ax.set_xticks(x)
    ax.set_xticklabels(df["test_label"], rotation=45, ha="right")
    ax.set_ylim(0, 1.5)

    # 6. Метрики графа
    ax = axes[1, 2]
    if "network_diameter" in df.columns and "avg_path_length" in df.columns:
        d = df["network_diameter"].replace(-1, np.nan)
        p = df["avg_path_length"].replace(-1, np.nan)
        ax.plot(x, d, "o-", label="Диаметр", color="blue")
        ax.plot(x, p, "s-", label="Ср. длина пути", color="green")
    ax.set_title("Характеристики графа")
    ax.set_ylabel("значение")
    ax.set_xticks(x)
    ax.set_xticklabels(df["test_label"], rotation=45, ha="right")
    ax.legend()

    plt.tight_layout()
    out_path = os.path.join(os.path.dirname(csv_file), "comparison_plots.png")
    plt.savefig(out_path, dpi=150, bbox_inches="tight")
    plt.close()
    print(f"Графики сохранены: {out_path}")

    print("\n📊 Сводная таблица:")
    cols = ["test_id", "detection_time", "alert_coverage", "peak_load", "evil_reputation_after", "successful_attack"]
    cols = [c for c in cols if c in df.columns]
    print(df[cols].to_string(index=False))


if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Использование: python3 plot_results.py <path_to_results.csv>")
        sys.exit(1)
    plot_results(sys.argv[1])
