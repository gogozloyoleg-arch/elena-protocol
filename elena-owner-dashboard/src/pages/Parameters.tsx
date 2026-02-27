import { useState, useEffect } from 'react';
import { getNetworkParameters } from '../services/api';
import type { NetworkParams } from '../services/api';

const MICRO = 1_000_000;

function ParamRow({ label, value }: { label: string; value: string | number }) {
  return (
    <div className="param-row">
      <span className="param-label">{label}</span>
      <span className="param-value">{value}</span>
    </div>
  );
}

export function Parameters() {
  const [params, setParams] = useState<NetworkParams | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    setLoading(true);
    setError(null);
    getNetworkParameters()
      .then(setParams)
      .catch((e) => setError(e instanceof Error ? e.message : String(e)))
      .finally(() => setLoading(false));
  }, []);

  if (loading) {
    return (
      <div className="page">
        <h1>Параметры сети</h1>
        <div className="card">
          <p>Загрузка…</p>
        </div>
      </div>
    );
  }

  if (error || !params) {
    return (
      <div className="page">
        <h1>Параметры сети</h1>
        <div className="card">
          <p className="error">Не удалось загрузить параметры: {error ?? 'нет данных'}.</p>
          <p className="muted">Убедитесь, что gateway подключён к узлу elena-core с включённым admin RPC.</p>
        </div>
      </div>
    );
  }

  return (
    <div className="page">
      <h1>Параметры сети</h1>

      <div className="card">
        <h2>Эмиссия</h2>
        <ParamRow
          label="База эмиссии (на узел в час)"
          value={`${(params.emission_base_per_hour_micro / MICRO).toFixed(2)} ELENA`}
        />
        <ParamRow label="Примерный размер транзакции (байт)" value={params.approx_tx_bytes} />
        <ParamRow label="Примерный размер алерта (байт)" value={params.approx_alert_bytes} />
      </div>

      <div className="card">
        <h2>Комиссии</h2>
        <ParamRow
          label="Базовая комиссия"
          value={`${(params.fee_base_micro / MICRO).toFixed(4)} ELENA`}
        />
        <ParamRow label="Процент от суммы (basis points)" value={`${params.fee_rate_bp} (0.01%)`} />
        <ParamRow label="Доля хранителям" value={`${(params.fee_share_storage * 100).toFixed(0)}%`} />
        <ParamRow label="Доля ретрансляторам" value={`${(params.fee_share_relay * 100).toFixed(0)}%`} />
        <ParamRow label="Доля на сжигание" value={`${(params.fee_share_burn * 100).toFixed(0)}%`} />
      </div>

      <div className="card">
        <h2>Пороги</h2>
        <ParamRow
          label="Порог микроплатежа (ниже — возможна нулевая комиссия)"
          value={`${(params.micro_payment_threshold_micro / MICRO).toFixed(2)} ELENA`}
        />
        <ParamRow
          label="Репутация для бесплатных микроплатежей"
          value={params.free_micro_reputation}
        />
        <ParamRow
          label="Максимальное предложение"
          value={`${(params.max_supply_micro / MICRO / 1_000_000).toFixed(0)} млн ELENA`}
        />
      </div>

      <div className="card">
        <h2>Репутация</h2>
        <ParamRow label="Минимальная репутация после наказания" value={params.reputation_punish_min} />
        <ParamRow
          label="Прирост за хранение (за день)"
          value={params.reputation_delta_storage_per_day}
        />
        <ParamRow label="Прирост за ретрансляцию" value={params.reputation_delta_relay} />
        <ParamRow label="Прирост за алерт" value={params.reputation_delta_alert} />
        <ParamRow
          label="Снижение за бездействие >30 дней (за день)"
          value={params.reputation_decay_inactive_per_day}
        />
      </div>

      <div className="card">
        <h2>Двойная трата</h2>
        <ParamRow
          label="Порог баланса для сжигания 1% (ELENA)"
          value={params.double_spend_burn_threshold_elena}
        />
        <ParamRow
          label="Процент сжигания при двойной трате"
          value={`${(params.double_spend_burn_pct * 100).toFixed(0)}%`}
        />
      </div>

      <div className="card muted">
        <p>
          Изменение параметров (базовая эмиссия, комиссии, пороги) будет доступно через голосование по репутации.
        </p>
        <p>Сейчас параметры задаются в elena-core (economics, node config).</p>
      </div>
    </div>
  );
}
