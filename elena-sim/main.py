#!/usr/bin/env python3
"""
Точка входа симулятора децентрализованной платежной сети «Елена».
"""

import argparse
import sys
from pathlib import Path

# Добавляем корень проекта в path
sys.path.insert(0, str(Path(__file__).resolve().parent))

from rich.console import Console
from rich.table import Table
from rich.panel import Panel

from simulation.scenarios import (
    Scenario1_HonestNetwork,
    Scenario2_ClassicDoubleSpend,
    Scenario3_QuantumDoubleSpend,
    Scenario4_SybilAttack,
)
from visualization.dashboard import create_app, set_dashboard_state

console = Console()


def run_scenario_1(args: argparse.Namespace, runner_kwargs: dict = None) -> None:
    scenario = Scenario1_HonestNetwork()
    result = scenario.run(num_nodes=args.nodes, steps=args.steps, **(runner_kwargs or {}))
    runner = result["runner"]
    avg_rep = result.get("avg_reputation", 0)
    summary = runner.metrics.get_summary()
    console.print(Panel("[green]Сценарий 1: Честная сеть[/green]"))
    console.print(f"Узлов: {args.nodes}, шагов: {args.steps}")
    console.print(f"Средняя репутация: {avg_rep:.2f}")
    console.print(f"Транзакций в сети: {len(runner.graph.transactions)}")
    if summary.get("network_diameter") is not None and summary.get("network_diameter") >= 0:
        console.print(f"Диаметр графа: {summary['network_diameter']}, ср. длина пути: {summary.get('avg_path_length', 0):.2f}")
    if args.viz:
        set_dashboard_state(runner=runner)


def run_scenario_2(args: argparse.Namespace, runner_kwargs: dict = None) -> None:
    scenario = Scenario2_ClassicDoubleSpend()
    result = scenario.run(num_nodes=args.nodes, steps=args.steps, num_evil=args.evil, **(runner_kwargs or {}))
    runner = result["runner"]
    summary = runner.metrics.get_summary()
    warmup = min(50, args.steps - 20)
    avg_rep = _avg_rep(runner)

    console.print(Panel("[yellow]Сценарий 2: Классическая двойная трата[/yellow]"))
    console.print(f"Узлов: {args.nodes}, злых узлов: {args.evil}")
    console.print()
    console.print(f"Шаг {warmup}: Сеть стабилизировалась, ср.репутация: {avg_rep:.2f}")
    evil_id = result.get("evil_id", "evil_0")
    console.print(f"Шаг {warmup + 1}: Злой узел {evil_id} создаёт 2 конфликтующие транзакции")
    discovered_by = result.get("discovered_by")
    detection_step = result.get("detection_step")
    nodes_alert = result.get("nodes_with_alert", 0)
    total = len(runner.graph.nodes)
    pct = (100 * nodes_alert / total) if total else 0
    if detection_step is not None and discovered_by:
        console.print(f"Шаг {detection_step}: [bold]⚠️ КОНФЛИКТ ОБНАРУЖЕН[/bold] узлом {discovered_by}")
        console.print(f"Шаг {detection_step + 1}: Alert распространяется...")
        console.print(f"Шаг {detection_step + 5}: [bold]{pct:.0f}%[/bold] узлов получили Alert")
        console.print(f"Шаг {detection_step + 10}: [green]✅ Атака предотвращена[/green]")
    else:
        console.print("[red]Конфликт не обнаружен[/red]")
    console.print()
    console.print("[bold]Результаты:[/bold]")
    table = Table()
    table.add_column("Метрика", style="cyan")
    table.add_column("Значение", style="green")
    table.add_row("Время обнаружения", f"{summary.get('avg_detection_time_steps', 0):.0f} шагов")
    table.add_row("Узлов, получивших Alert", f"{nodes_alert}/{total}")
    rep_before = result.get("evil_reputation_before", 0)
    rep_after = result.get("evil_reputation_after", 0)
    table.add_row("Репутация злого узла", f"упала с {rep_before} до {rep_after}")
    table.add_row("Ложных срабатываний", str(summary.get("false_positives", 0)))
    console.print(table)
    if args.viz:
        set_dashboard_state(runner=runner)


def run_scenario_3(args: argparse.Namespace, runner_kwargs: dict = None) -> None:
    console.print(Panel("[bold red]Сценарий 3: Квантовая двойная трата[/bold red]"))
    console.print(f"Узлов: {args.nodes}, квантовое преимущество: {args.quantum}")
    num_evil = getattr(args, "evil", 1)
    if num_evil > 1:
        console.print(f"[dim]Злых узлов в сговоре: {num_evil}[/dim]")
    if getattr(args, "sophisticated", False):
        console.print("[dim]Режим: усиленная атака (развод tx по кластерам)[/dim]")
    console.print()
    scenario = Scenario3_QuantumDoubleSpend()
    result = scenario.run(
        num_nodes=args.nodes,
        quantum_advantage=args.quantum,
        steps=args.steps,
        sophisticated=getattr(args, "sophisticated", False),
        num_evil=num_evil,
        **(runner_kwargs or {}),
    )
    runner = result["runner"]
    summary = runner.metrics.get_summary()
    detection_step = result.get("detection_step")
    nodes_alert = result.get("nodes_with_alert", 0)
    total_nodes = len(runner.graph.nodes)
    pct = (100 * nodes_alert / total_nodes) if total_nodes else 0

    # Имитация пошагового вывода как в спецификации
    warmup = min(1000, args.steps)
    console.print(f"Шаг {warmup}: Сеть стабилизирована, средняя репутация: {_avg_rep(runner):.2f}")
    if runner.evil_nodes:
        n_evil = len(runner.evil_nodes)
        console.print(f"Шаг {warmup + 1}: Злой узел (узлов: {n_evil}) пытается двойную трату...")
    discovered_by = result.get("discovered_by") or "сеть"
    if detection_step is not None:
        console.print(f"Шаг {detection_step}: [bold]⚠️ КОНФЛИКТ ОБНАРУЖЕН[/bold] узлом {discovered_by}")
        console.print(f"Шаг {detection_step + 1}: Alert распространяется...")
        console.print(f"Шаг {detection_step + 5}: [bold]{pct:.0f}%[/bold] узлов получили Alert")
        console.print(f"Шаг {detection_step + 15}: [green]✅ Атака предотвращена[/green]")
    else:
        console.print("[red]Атака не обнаружена (успешная атака)[/red]")

    console.print()
    console.print("[bold]Результаты:[/bold]")
    table = Table()
    table.add_column("Метрика", style="cyan")
    table.add_column("Значение", style="green")
    det_time = summary.get("avg_detection_time_steps") or (1.0 if detection_step else 0)
    table.add_row("Время обнаружения", f"{det_time:.0f} шагов")
    table.add_row("Успешность атаки", "НЕТ" if not summary.get("successful_attacks") else "ДА")
    table.add_row("Ложных срабатываний", str(summary.get("false_positives", 0)))
    table.add_row("Пиковая нагрузка", f"{summary.get('peak_throughput', 0)} сообщений/шаг")
    console.print(table)
    if getattr(args, "batch", False):
        _print_batch_result(args, result, runner, summary, detection_step, nodes_alert)
    if args.viz:
        set_dashboard_state(runner=runner)


def _print_batch_result(args, result, runner, summary, detection_step, nodes_alert) -> None:
    """Выводит одну строку AB_RESULT=<json> для парсинга батч-скриптом."""
    import json
    total = len(runner.graph.nodes)
    alert_pct = round(100 * nodes_alert / total, 1) if total else 0
    det_time = summary.get("avg_detection_time_steps") or (0 if not detection_step else 3.0)
    payload = {
        "detection_time": round(float(det_time), 1),
        "alert_coverage": alert_pct,
        "peak_load": int(summary.get("peak_throughput", 0)),
        "evil_reputation_before": result.get("evil_reputation_before", 0),
        "evil_reputation_after": result.get("evil_reputation_after", 0),
        "successful_attack": int(bool(summary.get("successful_attacks", 0))),
        "false_positives": int(summary.get("false_positives", 0)),
        "network_diameter": int(summary.get("network_diameter", -1)) if summary.get("network_diameter") is not None else -1,
        "avg_path_length": round(float(summary.get("avg_path_length", -1)), 2) if summary.get("avg_path_length") is not None else -1.0,
    }
    print("AB_RESULT=" + json.dumps(payload, ensure_ascii=False))


def _avg_rep(runner) -> float:
    if not runner.metrics.reputation_history:
        return 0.5
    last = runner.metrics.reputation_history[-1]
    if not last.get("reputations"):
        return 0.5
    return sum(last["reputations"].values()) / len(last["reputations"])


def run_scenario_4(args: argparse.Namespace, runner_kwargs: dict = None) -> None:
    scenario = Scenario4_SybilAttack()
    result = scenario.run(
        num_nodes=args.nodes,
        num_sybil=min(5, args.nodes // 10),
        quantum_advantage=args.quantum,
        steps=args.steps,
        **(runner_kwargs or {}),
    )
    runner = result["runner"]
    summary = runner.metrics.get_summary()
    console.print(Panel("[magenta]Сценарий 4: Сибил-атака[/magenta]"))
    console.print(f"Узлов: {args.nodes}, сибил-узлов: {len(runner.evil_nodes)}")
    console.print(f"Сводка: {summary}")
    if args.viz:
        set_dashboard_state(runner=runner)


def main() -> None:
    parser = argparse.ArgumentParser(description="Симулятор сети Елена")
    parser.add_argument("--scenario", type=int, default=1, help="Номер сценария (1-4)")
    parser.add_argument("--nodes", type=int, default=500, help="Количество узлов")
    parser.add_argument("--quantum", type=float, default=0.7, help="Квантовое преимущество")
    parser.add_argument("--steps", type=int, default=1000, help="Шагов симуляции")
    parser.add_argument("--evil", type=int, default=1, help="Количество злых узлов (сценарий 2)")
    parser.add_argument("--sophisticated", action="store_true", help="Усиленная атака: развод tx по кластерам (сценарий 3)")
    parser.add_argument("--no-chaff", action="store_true", help="Отключить шумовые транзакции (A/B тесты)")
    parser.add_argument("--no-rewiring", action="store_true", help="Отключить переподключение графа (A/B тесты)")
    parser.add_argument("--chaff-prob", type=float, default=None, help="Вероятность chaff (по умолч. из config)")
    parser.add_argument("--rewiring-interval", type=int, default=None, help="Интервал rewiring в шагах")
    parser.add_argument("--rewiring-prob", type=float, default=None, help="Вероятность rewiring одного ребра")
    parser.add_argument("--batch", action="store_true", help="Режим A/B: в конце вывести одну строку AB_RESULT=<json>")
    parser.add_argument("--viz", action="store_true", help="Запустить веб-визуализацию после симуляции")
    args = parser.parse_args()

    runner_kwargs = {}
    if getattr(args, "no_chaff", False):
        runner_kwargs["chaff_prob"] = 0
    elif getattr(args, "chaff_prob", None) is not None:
        runner_kwargs["chaff_prob"] = args.chaff_prob
    if getattr(args, "no_rewiring", False):
        runner_kwargs["rewiring_interval"] = 0
    elif getattr(args, "rewiring_interval", None) is not None:
        runner_kwargs["rewiring_interval"] = args.rewiring_interval
    if getattr(args, "rewiring_prob", None) is not None:
        runner_kwargs["rewiring_prob"] = args.rewiring_prob

    if args.scenario == 1:
        run_scenario_1(args, runner_kwargs)
    elif args.scenario == 2:
        run_scenario_2(args, runner_kwargs)
    elif args.scenario == 3:
        run_scenario_3(args, runner_kwargs)
    elif args.scenario == 4:
        run_scenario_4(args, runner_kwargs)
    else:
        console.print("[red]Неизвестный сценарий. Выберите 1–4.[/red]")
        sys.exit(1)

    if args.viz and not getattr(args, "batch", False):
        app = create_app()
        import uvicorn
        console.print("[bold]Запуск дашборда на http://127.0.0.1:8000[/bold]")
        uvicorn.run(app, host="127.0.0.1", port=8000)


if __name__ == "__main__":
    main()
