const API_URL = import.meta.env.VITE_API_URL || 'http://localhost:9180';

/** Список URL gateway'ев для мониторинга узлов. Через запятую в VITE_GATEWAY_URLS или один VITE_API_URL. */
export function getGatewayUrls(): string[] {
  const urls = import.meta.env.VITE_GATEWAY_URLS;
  if (urls && typeof urls === 'string') {
    return urls.split(',').map((s: string) => s.trim()).filter(Boolean);
  }
  return [API_URL];
}

export type NetworkStats = {
  peer_id: string;
  balance: number;
  reputation: Record<string, number>;
  transactions: number;
  alerts: number;
};

/** Параметры сети (economics из elena-core). */
export type NetworkParams = {
  micro_per_elena: number;
  max_supply_micro: number;
  fee_base_micro: number;
  fee_rate_bp: number;
  micro_payment_threshold_micro: number;
  free_micro_reputation: number;
  fee_share_storage: number;
  fee_share_relay: number;
  fee_share_burn: number;
  reputation_punish_min: number;
  double_spend_burn_threshold_elena: number;
  double_spend_burn_pct: number;
  reputation_delta_storage_per_day: number;
  reputation_delta_relay: number;
  reputation_delta_alert: number;
  reputation_decay_inactive_per_day: number;
  emission_base_per_hour_micro: number;
  approx_tx_bytes: number;
  approx_alert_bytes: number;
};

export async function getStatus(): Promise<{ status: string; node: string }> {
  const r = await fetch(`${API_URL}/api/v1/status`);
  if (!r.ok) throw new Error(await r.text());
  return r.json();
}

export async function getNetworkStats(): Promise<NetworkStats> {
  const r = await fetch(`${API_URL}/api/v1/network/stats`);
  if (!r.ok) throw new Error(await r.text());
  return r.json();
}

export async function getStatusFrom(baseUrl: string): Promise<{ status: string; node: string }> {
  const r = await fetch(`${baseUrl.replace(/\/$/, '')}/api/v1/status`);
  if (!r.ok) throw new Error(await r.text());
  return r.json();
}

export async function getNetworkStatsFrom(baseUrl: string): Promise<NetworkStats> {
  const r = await fetch(`${baseUrl.replace(/\/$/, '')}/api/v1/network/stats`);
  if (!r.ok) throw new Error(await r.text());
  return r.json();
}

export async function getNetworkParameters(): Promise<NetworkParams> {
  const r = await fetch(`${API_URL}/api/v1/network/parameters`);
  if (!r.ok) throw new Error(await r.text());
  return r.json();
}
