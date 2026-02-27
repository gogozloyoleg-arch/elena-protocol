"""Параметры конфигурации симуляции сети Елена."""

SIMULATION_PARAMS = {
    "num_nodes": 500,
    "num_evil_nodes": 1,
    "quantum_advantage": 0.7,
    "chaff_probability": 0,  # по умолчанию без chaff (оптимально по нагрузке; для шума задать в CLI)
    "rewiring_interval": 100,
    "max_steps": 10000,
    "tx_per_step": 10,
    "reputation_threshold": 0.8,  # для арбитров
    "confidence_threshold": 0.99,  # для финальности
    "alert_priority_multiplier": 10,  # приоритет алертов
    "initial_balance": 1000.0,
    "peer_degree_min": 3,
    "peer_degree_max": 10,
}

REPUTATION_PARAMS = {
    "initial_reputation": 0.5,
    "reward_per_tx_forwarded": 0.001,  # +0.001 за каждую пересланную транзакцию
    "reward_per_alert_propagated": 0.01,  # +0.01 за распространение алерта
    "decay_per_step": 0.0001,  # небольшая естественная убыль
    "max_reputation": 0.99,
    "min_reputation": 0.01,
    "penalty_double_spend": 0.2,  # снижение репутации за двойную трату (за один алерт)
}

METRICS_TO_COLLECT = {
    "avg_reputation": [],
    "reputation_distribution": [],
    "tx_confidence_5": [],
    "tx_confidence_10": [],
    "tx_confidence_20": [],
    "alert_propagation_time": [],
    "false_positive_rate": 0,
    "network_diameter": 0,
    "avg_path_length": 0,
}
