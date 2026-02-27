const DEFAULT_API_URL = import.meta.env.VITE_API_URL || 'http://localhost:9180';
const API_URL_STORAGE = 'elena_api_url';
const API_KEY_STORAGE = 'elena_api_key';

/** URL gateway (из настроек или дефолт) */
export function getApiUrl(): string {
  try {
    const url = sessionStorage.getItem(API_URL_STORAGE)?.trim();
    return url && url.startsWith('http') ? url.replace(/\/$/, '') : DEFAULT_API_URL;
  } catch {
    return DEFAULT_API_URL;
  }
}

export function setApiUrl(url: string | null): void {
  try {
    if (!url || !url.trim()) sessionStorage.removeItem(API_URL_STORAGE);
    else sessionStorage.setItem(API_URL_STORAGE, url.trim().replace(/\/$/, ''));
  } catch {
    /* ignore */
  }
}

/** Сохранённый URL (null = используется дефолт) */
export function getStoredApiUrl(): string | null {
  try {
    const url = sessionStorage.getItem(API_URL_STORAGE)?.trim();
    return url && url.startsWith('http') ? url.replace(/\/$/, '') : null;
  } catch {
    return null;
  }
}

export function getApiKey(): string | null {
  try {
    return sessionStorage.getItem(API_KEY_STORAGE);
  } catch {
    return null;
  }
}

export function setApiKey(key: string | null): void {
  try {
    if (key === null || key === '') sessionStorage.removeItem(API_KEY_STORAGE);
    else sessionStorage.setItem(API_KEY_STORAGE, key.trim());
  } catch {
    /* ignore */
  }
}

function authHeaders(): Record<string, string> {
  const key = getApiKey();
  if (!key) return {};
  return { 'X-API-Key': key };
}

export type NetworkStats = {
  peer_id: string;
  balance: number;
  reputation: Record<string, number>;
  transactions: number;
  alerts: number;
};

export async function getStatus(): Promise<{ status: string; node: string }> {
  const r = await fetch(`${getApiUrl()}/api/v1/status`);
  if (!r.ok) throw new Error(await r.text());
  return r.json();
}

export async function getNetworkStats(): Promise<NetworkStats> {
  const r = await fetch(`${getApiUrl()}/api/v1/network/stats`);
  if (!r.ok) throw new Error(await r.text());
  return r.json();
}

export async function getPubkey(): Promise<{ pubkey: string }> {
  const r = await fetch(`${getApiUrl()}/api/v1/wallet/pubkey`);
  if (!r.ok) throw new Error(await r.text());
  return r.json();
}

export type RecentTxItem = {
  id: string;
  from: string;
  to: string;
  amount: number;
  timestamp: number;
};

export async function getRecentTransactions(limit = 20): Promise<RecentTxItem[]> {
  const r = await fetch(`${getApiUrl()}/api/v1/transactions/recent?limit=${limit}`);
  if (!r.ok) return [];
  const data = await r.json();
  return Array.isArray(data) ? data : [];
}

export async function sendTransaction(to: string, amount: number): Promise<{ tx_id: string }> {
  const r = await fetch(`${getApiUrl()}/api/v1/transaction/send`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json', ...authHeaders() },
    body: JSON.stringify({ to: to.trim(), amount }),
  });
  const data = await r.json().catch(() => ({}));
  if (r.status === 401) throw new Error('API_KEY_REQUIRED');
  if (!r.ok) throw new Error((data as { error?: string }).error || 'Send failed');
  return data as { tx_id: string };
}

export async function stake(amount: number): Promise<{ ok: boolean }> {
  const r = await fetch(`${getApiUrl()}/api/v1/wallet/stake`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json', ...authHeaders() },
    body: JSON.stringify({ amount }),
  });
  const data = await r.json().catch(() => ({}));
  if (r.status === 401) throw new Error('API_KEY_REQUIRED');
  if (!r.ok) throw new Error((data as { error?: string }).error || 'Stake failed');
  return data as { ok: boolean };
}
