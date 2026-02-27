"""
Сценарии симуляции: честная сеть, классическая и квантовая двойная трата, Сибил-атака.
"""

import random
from typing import List

from core import Node, QuantumEvilNode, NetworkGraph
from config import SIMULATION_PARAMS
from .runner import SimulationRunner
from .metrics import MetricsCollector


class Scenario1_HonestNetwork:
    """Базовая честная сеть без атак."""

    def run(self, num_nodes: int = 500, steps: int = 1000, **runner_kwargs) -> dict:
        runner = SimulationRunner(num_nodes=num_nodes, num_evil=0, **runner_kwargs)
        runner.build_network()
        for step in range(steps):
            runner.step(step)
        summary = runner.metrics.get_summary()
        avg_rep = 0.0
        if runner.metrics.reputation_history:
            last = runner.metrics.reputation_history[-1]
            avg_rep = sum(last["reputations"].values()) / len(last["reputations"]) if last["reputations"] else 0
        return {
            "summary": summary,
            "avg_reputation": avg_rep,
            "runner": runner,
        }


class Scenario2_ClassicDoubleSpend:
    """Классическая двойная трата (злой узел без квантового преимущества)."""

    def run(
        self,
        num_nodes: int = 500,
        steps: int = 200,
        num_evil: int = 1,
        **runner_kwargs,
    ) -> dict:
        runner = SimulationRunner(
            num_nodes=num_nodes,
            num_evil=num_evil,
            quantum_advantage=0.0,
            **runner_kwargs,
        )
        runner.build_network()
        warmup = min(50, steps - 20)
        for step in range(warmup):
            runner.step(step)
        evil = runner.evil_nodes[0] if runner.evil_nodes else None
        evil_rep_before = round(evil.reputation, 2) if evil else 0
        evil_rep_after = evil_rep_before
        discovered_by = None
        nodes_with_alert = 0
        detection_step_val = None

        if not evil:
            for step in range(warmup, steps):
                runner.step(step)
            return {"summary": runner.metrics.get_summary(), "runner": runner}

        targets = [n.id for n in runner.honest_nodes[:2] if n.id != evil.id]
        if len(targets) < 2:
            targets = [nid for nid in list(runner.graph.nodes.keys())[:5] if nid != evil.id][:2]
        tx1, tx2 = evil.double_spend_attack(targets[0], targets[1], min(100.0, evil.balance / 2))
        if tx1 and tx2:
            runner.metrics.alerts_created += 1
            runner.graph.propagate_transaction(tx1, evil)
            runner.graph.propagate_transaction(tx2, evil)
            for n in runner.graph.nodes.values():
                if tx1.id in n.conflicting_tx_ids or tx2.id in n.conflicting_tx_ids:
                    nodes_with_alert += 1
                    if discovered_by is None:
                        discovered_by = n.id
                    if detection_step_val is None:
                        detection_step_val = warmup + 2
            if nodes_with_alert > 0:
                runner.metrics.record_detection(2.0)
                runner.metrics.record_attack_result(False)
                if runner.graph.alerts:
                    discovered_by = next(iter(runner.graph.alerts.values())).discovered_by
            else:
                runner.metrics.record_attack_result(True)
            evil_rep_after = round(evil.reputation, 2)

        for step in range(warmup, steps):
            runner.step(step)

        return {
            "summary": runner.metrics.get_summary(),
            "runner": runner,
            "evil_id": evil.id,
            "evil_reputation_before": evil_rep_before,
            "evil_reputation_after": evil_rep_after,
            "discovered_by": discovered_by,
            "nodes_with_alert": nodes_with_alert,
            "detection_step": detection_step_val,
        }


class Scenario3_QuantumDoubleSpend:
    """Квантовая двойная трата: один или несколько злых узлов (в сговоре — атака на одном шаге)."""

    def run(
        self,
        num_nodes: int = 500,
        quantum_advantage: float = 0.7,
        steps: int = 1000,
        sophisticated: bool = False,
        num_evil: int = 1,
        **runner_kwargs,
    ) -> dict:
        runner = SimulationRunner(
            num_nodes=num_nodes,
            num_evil=num_evil,
            quantum_advantage=quantum_advantage,
            **runner_kwargs,
        )
        runner.build_network()
        for step in range(min(1000, steps)):
            runner.step(step)
        if not runner.evil_nodes:
            return {"summary": runner.metrics.get_summary(), "runner": runner,
                    "detection_step": None, "nodes_with_alert": 0, "discovered_by": None,
                    "evil_reputation_before": 0, "evil_reputation_after": 0}
        attack_step = min(1000, steps - 1)
        first_evil = runner.evil_nodes[0]
        evil_rep_before = round(first_evil.reputation, 2)
        evil_rep_after = evil_rep_before
        detection_step = None
        nodes_with_alert_set = set()
        honest_ids = [n.id for n in runner.honest_nodes]
        for i, evil in enumerate(runner.evil_nodes):
            idx = (i * 2) % max(len(honest_ids), 1)
            targets = [honest_ids[idx % len(honest_ids)], honest_ids[(idx + 1) % len(honest_ids)]]
            if len(targets) < 2 or targets[0] == targets[1]:
                targets = [nid for nid in list(runner.graph.nodes.keys()) if nid != evil.id][:2]
            if len(targets) < 2:
                continue
            amount = min(100.0, evil.balance / 2)
            if sophisticated:
                tx1, tx2 = evil.sophisticated_double_spend(targets[0], targets[1], amount, runner.graph)
            else:
                tx1, tx2 = evil.double_spend_attack(targets[0], targets[1], amount)
            if tx1 and tx2:
                runner.metrics.alerts_created += 1
                runner.graph.propagate_transaction(tx1, evil)
                runner.graph.propagate_transaction(tx2, evil)
                for n in runner.graph.nodes.values():
                    if tx1.id in n.conflicting_tx_ids or tx2.id in n.conflicting_tx_ids:
                        if detection_step is None:
                            detection_step = attack_step + 3
                        nodes_with_alert_set.add(n.id)
                if detection_step is not None:
                    runner.metrics.record_detection(float(detection_step - attack_step))
                    runner.metrics.record_attack_result(False)
                    runner.metrics.nodes_received_alert.append(len(nodes_with_alert_set))
                else:
                    runner.metrics.record_attack_result(True)
                evil_rep_after = round(evil.reputation, 2)
        nodes_with_alert = len(nodes_with_alert_set)
        for step in range(attack_step + 1, steps):
            runner.step(step)
        if runner.evil_nodes:
            evil_rep_after = round(runner.evil_nodes[0].reputation, 2)
        discovered_by = None
        if runner.graph.alerts:
            discovered_by = next(iter(runner.graph.alerts.values())).discovered_by
        return {
            "summary": runner.metrics.get_summary(),
            "runner": runner,
            "detection_step": detection_step,
            "nodes_with_alert": nodes_with_alert,
            "discovered_by": discovered_by,
            "evil_reputation_before": evil_rep_before,
            "evil_reputation_after": evil_rep_after,
        }


class Scenario4_SybilAttack:
    """Сибил-атака с квантовым усилением: несколько злых узлов."""

    def run(
        self,
        num_nodes: int = 500,
        num_sybil: int = 5,
        quantum_advantage: float = 0.7,
        steps: int = 500,
        **runner_kwargs,
    ) -> dict:
        runner = SimulationRunner(
            num_nodes=num_nodes + num_sybil,
            num_evil=num_sybil,
            quantum_advantage=quantum_advantage,
            **runner_kwargs,
        )
        runner.build_network()
        for step in range(steps):
            runner.step(step)
        return {"summary": runner.metrics.get_summary(), "runner": runner}
