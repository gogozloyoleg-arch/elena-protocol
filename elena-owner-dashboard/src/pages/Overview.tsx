import { useState, useEffect } from 'react';
import { getStatus, getNetworkStats } from '../services/api';
import type { NetworkStats } from '../services/api';

const MICRO = 1_000_000;

export function Overview() {
  const [status, setStatus] = useState<{ status: string; node: string } | null>(null);
  const [stats, setStats] = useState<NetworkStats | null>(null);

  useEffect(() => {
    getStatus().then(setStatus).catch(() => setStatus({ status: 'error', node: '' }));
    getNetworkStats().then(setStats).catch(() => setStats(null));
  }, []);

  return (
    <div className="page">
      <h1>ELENA NETWORK DASHBOARD <span className="badge">OWNER</span></h1>
      <section className="section">
        <h2>Статистика сети</h2>
        <div className="cards">
          <div className="card">
            <h3>Узел</h3>
            <p>{status?.status === 'ok' ? '🟢 Онлайн' : status?.status || '—'}</p>
            <p className="muted">{status?.node}</p>
          </div>
          {stats && (
            <>
              <div className="card">
                <h3>Транзакций</h3>
                <p className="big">{stats.transactions}</p>
              </div>
              <div className="card">
                <h3>Алертов</h3>
                <p className="big">{stats.alerts}</p>
              </div>
              <div className="card">
                <h3>Σ Баланс (этот узел)</h3>
                <p className="big">{(stats.balance / MICRO).toFixed(2)} ELENA</p>
              </div>
            </>
          )}
        </div>
      </section>
      <section className="section">
        <h2>Репутация узла</h2>
        {stats && (
          <pre className="code">{JSON.stringify(stats.reputation, null, 2)}</pre>
        )}
      </section>
    </div>
  );
}
