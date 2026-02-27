//! Параметры и формулы экономики протокола «Елена» (Tokenomics v1.0).
//!
//! Единицы: балансы и суммы в **микро-ELENA** (1 ELENA = 1_000_000 единиц).
//! Репутация: f64 в диапазоне [0.01, 0.99].

use serde::Serialize;

/// 1 ELENA = 1_000_000 микро-единиц (для комиссий 0.0001 ELENA и т.д.)
pub const MICRO_PER_ELENA: u64 = 1_000_000;

/// Максимальное предложение: 21 млн ELENA (в микро-единицах)
pub const MAX_SUPPLY_MICRO: u64 = 21_000_000 * MICRO_PER_ELENA;

/// Базовая комиссия: 0.0001 ELENA (защита от спама)
pub const FEE_BASE_MICRO: u64 = 100; // 0.0001 * MICRO_PER_ELENA

/// Процент от суммы: 0.01% = 1/10000
pub const FEE_RATE_BP: u64 = 1; // basis points (0.01%)

/// Порог микроплатежа: ниже этой суммы комиссия может быть 0 при высокой репутации (0.01 ELENA)
pub const MICRO_PAYMENT_THRESHOLD_MICRO: u64 = 10_000; // 0.01 ELENA

/// Репутация, при которой микроплатежи бесплатны
pub const FREE_MICRO_REPUTATION: f64 = 0.8;

/// Доля комиссии хранителям транзакций
pub const FEE_SHARE_STORAGE: f64 = 0.5;
/// Доля комиссии ретрансляторам
pub const FEE_SHARE_RELAY: f64 = 0.3;
/// Доля комиссии на сжигание
pub const FEE_SHARE_BURN: f64 = 0.2;

/// Минимальная репутация после наказания за двойную трату
pub const REPUTATION_PUNISH_MIN: f64 = 0.01;
/// Порог баланса (ELENA), выше которого при двойной трате сжигается 1%
pub const DOUBLE_SPEND_BURN_THRESHOLD_ELENA: u64 = 100;
/// Процент сжигания баланса при двойной трате
pub const DOUBLE_SPEND_BURN_PCT: f64 = 0.01;

/// Прирост репутации за хранение транзакций (за день)
pub const REPUTATION_DELTA_STORAGE_PER_DAY: f64 = 0.001;
/// Прирост репутации за распространение транзакции
pub const REPUTATION_DELTA_RELAY: f64 = 0.0005;
/// Прирост репутации за создание Alert'а
pub const REPUTATION_DELTA_ALERT: f64 = 0.01;
/// Снижение репутации за бездействие >30 дней (за день)
pub const REPUTATION_DECAY_INACTIVE_PER_DAY: f64 = 0.001;

/// База эмиссии: 1 ELENA в час на узел (упрощённая модель без нормализации по сети)
pub const EMISSION_BASE_PER_HOUR_MICRO: u64 = MICRO_PER_ELENA;
/// Примерный размер одной транзакции в байтах (для расчёта награды за хранение)
pub const APPROX_TX_BYTES: usize = 3000;
/// Примерный размер одного алерта в байтах
pub const APPROX_ALERT_BYTES: usize = 500;

/// Параметры сети для API (текущие значения из economics).
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct NetworkParams {
    pub micro_per_elena: u64,
    pub max_supply_micro: u64,
    pub fee_base_micro: u64,
    pub fee_rate_bp: u64,
    pub micro_payment_threshold_micro: u64,
    pub free_micro_reputation: f64,
    pub fee_share_storage: f64,
    pub fee_share_relay: f64,
    pub fee_share_burn: f64,
    pub reputation_punish_min: f64,
    pub double_spend_burn_threshold_elena: u64,
    pub double_spend_burn_pct: f64,
    pub reputation_delta_storage_per_day: f64,
    pub reputation_delta_relay: f64,
    pub reputation_delta_alert: f64,
    pub reputation_decay_inactive_per_day: f64,
    pub emission_base_per_hour_micro: u64,
    pub approx_tx_bytes: usize,
    pub approx_alert_bytes: usize,
}

/// Возвращает текущие параметры сети (для RPC/API).
pub fn network_params() -> NetworkParams {
    NetworkParams {
        micro_per_elena: MICRO_PER_ELENA,
        max_supply_micro: MAX_SUPPLY_MICRO,
        fee_base_micro: FEE_BASE_MICRO,
        fee_rate_bp: FEE_RATE_BP,
        micro_payment_threshold_micro: MICRO_PAYMENT_THRESHOLD_MICRO,
        free_micro_reputation: FREE_MICRO_REPUTATION,
        fee_share_storage: FEE_SHARE_STORAGE,
        fee_share_relay: FEE_SHARE_RELAY,
        fee_share_burn: FEE_SHARE_BURN,
        reputation_punish_min: REPUTATION_PUNISH_MIN,
        double_spend_burn_threshold_elena: DOUBLE_SPEND_BURN_THRESHOLD_ELENA,
        double_spend_burn_pct: DOUBLE_SPEND_BURN_PCT,
        reputation_delta_storage_per_day: REPUTATION_DELTA_STORAGE_PER_DAY,
        reputation_delta_relay: REPUTATION_DELTA_RELAY,
        reputation_delta_alert: REPUTATION_DELTA_ALERT,
        reputation_decay_inactive_per_day: REPUTATION_DECAY_INACTIVE_PER_DAY,
        emission_base_per_hour_micro: EMISSION_BASE_PER_HOUR_MICRO,
        approx_tx_bytes: APPROX_TX_BYTES,
        approx_alert_bytes: APPROX_ALERT_BYTES,
    }
}

/// Сериализует параметры сети в JSON (для admin RPC «params»).
pub fn network_params_json() -> String {
    serde_json::to_string(&network_params()).unwrap_or_else(|_| "{}".into())
}

/// Множитель приоритета транзакции для комиссии
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum TxPriority {
    Normal = 1,
    Urgent = 2,
    Critical = 10,
}

impl TxPriority {
    pub fn multiplier(self) -> u64 {
        self as u64
    }
}

impl Default for TxPriority {
    fn default() -> Self {
        TxPriority::Normal
    }
}

/// Вычисляет комиссию за транзакцию (в микро-ELENA).
///
/// Формула: `база + (сумма × 0.0001) × приоритет`.
/// Микроплатежи (< 0.01 ELENA) при репутации отправителя >= 0.8 — бесплатно (0).
pub fn compute_fee_micro(
    amount_micro: u64,
    priority: TxPriority,
    sender_reputation: f64,
) -> u64 {
    if amount_micro < MICRO_PAYMENT_THRESHOLD_MICRO && sender_reputation >= FREE_MICRO_REPUTATION {
        return 0;
    }
    let variable = (amount_micro * FEE_RATE_BP / 10_000).saturating_mul(priority.multiplier());
    (FEE_BASE_MICRO + variable).max(FEE_BASE_MICRO)
}

/// Фактор репутации для эмиссии: от 0.5 до 2.0.
/// `reputation` в [0.01, 0.99].
pub fn emission_reputation_factor(reputation: f64) -> f64 {
    let r = reputation.clamp(0.01, 0.99);
    0.5 + 1.5 * (r - 0.01) / 0.98 // линейно от 0.5 до 2.0
}

/// Эффективная репутация при стейкинге: `r × (1 + 0.5 × доля_замороженной)`.
/// `stake_fraction` в [0.0, 0.5].
pub fn effective_reputation_staked(base_reputation: f64, stake_fraction: f64) -> f64 {
    let s = stake_fraction.clamp(0.0, 0.5);
    (base_reputation * (1.0 + 0.5 * s)).min(0.99)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fee_normal() {
        // 1 ELENA, normal priority: 100 + 1_000_000/10000*1 = 100 + 100 = 200 micro
        let fee = compute_fee_micro(MICRO_PER_ELENA, TxPriority::Normal, 0.5);
        assert_eq!(fee, 200);
    }

    #[test]
    fn test_fee_micro_free_high_rep() {
        // 0.005 ELENA = 5000 micro < 10000, rep 0.9 -> free
        let fee = compute_fee_micro(5_000, TxPriority::Normal, 0.9);
        assert_eq!(fee, 0);
    }

    #[test]
    fn test_fee_micro_not_free_low_rep() {
        let fee = compute_fee_micro(5_000, TxPriority::Normal, 0.5);
        assert!(fee >= FEE_BASE_MICRO);
    }
}
