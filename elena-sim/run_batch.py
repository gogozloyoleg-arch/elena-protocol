#!/usr/bin/env python3
"""
Пакетный запуск A/B тестов для сети «Елена».
Сохраняет результаты в results/ab_tests_<date>/results.csv и логи в logs/.
Запуск: python3 run_batch.py [--output-dir results/ab_tests_YYYYMMDD_HHMMSS]
"""

import subprocess
import csv
import json
import re
import os
import sys
import argparse
from datetime import datetime
from typing import Dict, List, Optional, Any

# Корень проекта (где main.py)
PROJECT_ROOT = os.path.dirname(os.path.abspath(__file__))


class ABTester:
    def __init__(
        self,
        nodes: int = 100,
        steps: int = 150,
        quantum: float = 0.9,
        scenario: int = 3,
        output_dir: Optional[str] = None,
    ):
        self.nodes = nodes
        self.steps = steps
        self.quantum = quantum
        # Масштаб по умолчанию: 200 узлов, 500 шагов (можно переопределить через CLI)
        self.scenario = scenario
        self.timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        self.output_dir = output_dir or os.path.join(PROJECT_ROOT, "results", f"ab_tests_{self.timestamp}")
        os.makedirs(os.path.join(self.output_dir, "logs"), exist_ok=True)

    def run_test(
        self,
        test_id: str,
        chaff_prob: Optional[float] = 0.05,
        rewiring_interval: Optional[int] = 100,
        rewiring_prob: Optional[float] = 0.1,
    ) -> Dict[str, Any]:
        """Запускает один тест и возвращает метрики."""
        cmd = [
            sys.executable,
            os.path.join(PROJECT_ROOT, "main.py"),
            "--scenario", str(self.scenario),
            "--nodes", str(self.nodes),
            "--quantum", str(self.quantum),
            "--steps", str(self.steps),
            "--batch",
        ]
        if chaff_prob is None or chaff_prob == 0:
            cmd.append("--no-chaff")
            chaff_label = "off"
        else:
            cmd.extend(["--chaff-prob", str(chaff_prob)])
            chaff_label = "on"
        if rewiring_interval is None or rewiring_interval == 0:
            cmd.append("--no-rewiring")
            rewiring_label = "off"
        else:
            cmd.extend(["--rewiring-interval", str(rewiring_interval)])
            if rewiring_prob is not None:
                cmd.extend(["--rewiring-prob", str(rewiring_prob)])
            rewiring_label = "on"

        print(f"\n🚀 Запуск теста {test_id}")
        print(f"   Команда: {' '.join(cmd)}")

        log_path = os.path.join(self.output_dir, "logs", f"{test_id}.log")
        with open(log_path, "w", encoding="utf-8") as log_file:
            process = subprocess.run(
                cmd,
                cwd=PROJECT_ROOT,
                capture_output=True,
                text=True,
                encoding="utf-8",
                errors="replace",
            )
            log_file.write(process.stdout)
            if process.stderr:
                log_file.write("\n--- STDERR ---\n")
                log_file.write(process.stderr)

        output = process.stdout
        metrics: Dict[str, Any] = {
            "test_id": test_id,
            "chaff": chaff_label,
            "rewiring": rewiring_label,
            "chaff_prob": chaff_prob if chaff_prob is not None else 0,
            "rewiring_interval": rewiring_interval if rewiring_interval is not None else 0,
            "rewiring_prob": rewiring_prob if rewiring_prob is not None else 0,
        }

        # Парсим AB_RESULT=...
        match = re.search(r"AB_RESULT=(.+)", output)
        if match:
            try:
                data = json.loads(match.group(1).strip())
                metrics["detection_time"] = data.get("detection_time", 0)
                metrics["alert_coverage"] = data.get("alert_coverage", 0)
                metrics["peak_load"] = data.get("peak_load", 0)
                metrics["evil_reputation_before"] = data.get("evil_reputation_before", 0)
                metrics["evil_reputation_after"] = data.get("evil_reputation_after", 0)
                metrics["successful_attack"] = data.get("successful_attack", 0)
                metrics["false_positives"] = data.get("false_positives", 0)
                metrics["network_diameter"] = data.get("network_diameter", -1)
                metrics["avg_path_length"] = data.get("avg_path_length", -1)
            except json.JSONDecodeError:
                _fill_defaults(metrics)
        else:
            _fill_defaults(metrics)

        print(f"   ✅ Завершён. Время обнаружения: {metrics.get('detection_time', '?')} шагов")
        return metrics

    def run_all_tests(self) -> List[Dict[str, Any]]:
        """Запускает все тестовые конфигурации."""
        # baseline = рекомендуемая конфигурация (no_chaff — оптимально по нагрузке)
        configs = [
            ("baseline", None, 100, 0.1),           # без chaff, с rewiring (рекомендовано)
            ("with_chaff", 0.05, 100, 0.1),        # с шумом для сравнения
            ("no_rewiring", None, None, None),      # без rewiring
            ("no_protection", None, None, None),    # без защиты
            ("heavy_chaff", 0.15, 100, 0.1),
            ("frequent_rewiring", None, 50, 0.2),
            ("rare_rewiring", None, 200, 0.05),
            ("minimal", 0.01, 200, 0.02),
        ]
        if getattr(self, "_max_tests", None) is not None:
            configs = configs[: self._max_tests]
        results = []
        for test_id, chaff, rew_int, rew_prob in configs:
            row = self.run_test(test_id, chaff, rew_int, rew_prob)
            results.append(row)
            self._save_csv(results)
        self._print_summary(results)
        return results

    def _save_csv(self, results: List[Dict[str, Any]]) -> None:
        if not results:
            return
        path = os.path.join(self.output_dir, "results.csv")
        with open(path, "w", newline="", encoding="utf-8") as f:
            w = csv.DictWriter(f, fieldnames=results[0].keys())
            w.writeheader()
            w.writerows(results)
        print(f"   📁 Сохранено: {path}")

    def _print_summary(self, results: List[Dict[str, Any]]) -> None:
        print("\n" + "=" * 85)
        print("📊 СВОДНАЯ ТАБЛИЦА РЕЗУЛЬТАТОВ")
        print("=" * 85)
        print(f"{'Тест':<18} {'Chaff':<6} {'Rewiring':<9} {'Обнаруж.':<10} {'Alert %':<8} {'Нагрузка':<10} {'Репутация':<10}")
        print("-" * 85)
        for r in results:
            print(
                f"{r['test_id']:<18} {r['chaff']:<6} {r['rewiring']:<9} "
                f"{r.get('detection_time', 0):<10} {r.get('alert_coverage', 0):<8} "
                f"{r.get('peak_load', 0):<10} {r.get('evil_reputation_after', 0):<10}"
            )
        print("=" * 85)
        print(f"\n✅ Результаты: {os.path.join(self.output_dir, 'results.csv')}")
        print(f"   Графики: python3 plot_results.py \"{os.path.join(self.output_dir, 'results.csv')}\"")


def _fill_defaults(metrics: Dict[str, Any]) -> None:
    for key in ("detection_time", "alert_coverage", "peak_load", "evil_reputation_before",
                "evil_reputation_after", "successful_attack", "false_positives",
                "network_diameter", "avg_path_length"):
        if key not in metrics:
            metrics[key] = 0 if key != "avg_path_length" else -1.0


def main() -> None:
    parser = argparse.ArgumentParser(description="A/B тесты сети Елена")
    parser.add_argument("--nodes", type=int, default=200, help="Число узлов (масштаб)")
    parser.add_argument("--steps", type=int, default=500, help="Шагов симуляции")
    parser.add_argument("--quantum", type=float, default=0.9)
    parser.add_argument("--scale", choices=("small", "default", "large"), default="default",
                        help="small=50/80, default=200/500, large=300/800")
    parser.add_argument("--output-dir", type=str, default=None, help="Директория для results.csv и logs/")
    parser.add_argument("--max-tests", type=int, default=None, help="Макс. число тестов (для отладки)")
    args = parser.parse_args()
    nodes, steps = args.nodes, args.steps
    if getattr(args, "scale", None) == "small":
        nodes, steps = 50, 80
    elif getattr(args, "scale", None) == "large":
        nodes, steps = 300, 800
    tester = ABTester(nodes=nodes, steps=steps, quantum=args.quantum, output_dir=args.output_dir)
    tester._max_tests = getattr(args, "max_tests", None)
    print(f"Узлов: {tester.nodes}, шагов: {tester.steps}, quantum: {tester.quantum}")
    print(f"Результаты: {tester.output_dir}")
    tester.run_all_tests()


if __name__ == "__main__":
    main()
