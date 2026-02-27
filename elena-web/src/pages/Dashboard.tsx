import { useState, useEffect } from 'react';
import { Link } from 'react-router-dom';
import { getNetworkStats, getRecentTransactions } from '../services/api';
import { WalletCard } from '../components/WalletCard';
import { NetworkStatus } from '../components/NetworkStatus';
import type { NetworkStats, RecentTxItem } from '../services/api';

const MICRO = 1_000_000;

function formatTimeAgo(tsMs: number): string {
  const sec = Math.floor((Date.now() - tsMs) / 1000);
  if (sec < 60) return 'только что';
  if (sec < 3600) return `${Math.floor(sec / 60)} мин назад`;
  if (sec < 86400) return `${Math.floor(sec / 3600)} ч назад`;
  return `${Math.floor(sec / 86400)} дн назад`;
}

function shortHex(hex: string, n = 8): string {
  if (hex.length <= n) return hex;
  return hex.slice(0, n) + '…';
}

export function Dashboard() {
  const [stats, setStats] = useState<NetworkStats | null>(null);
  const [txs, setTxs] = useState<RecentTxItem[]>([]);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    getNetworkStats()
      .then(setStats)
      .catch((e) => setError(e.message));
    getRecentTransactions(15)
      .then(setTxs)
      .catch(() => setTxs([]));
  }, []);

  return (
    <div className="page">
      <header className="header">
        <h1>Кошелёк ELENA</h1>
        <NetworkStatus />
      </header>
      {error && <p className="error">{error}</p>}
      <div className="cards">
        <WalletCard stats={stats} />
      </div>
      <section className="section quick-start">
        <h3>Что можно сделать</h3>
        <ol className="list list-simple">
          <li><Link to="/receive">Получить</Link> — скопируйте свой адрес и отправьте его отправителю.</li>
          <li><Link to="/send">Отправить</Link> — вставьте адрес получателя и сумму в ELENA.</li>
          <li><Link to="/stake">Стейкинг</Link> — включите участие в сети и получайте долю вознаграждений.</li>
        </ol>
      </section>
      <nav className="nav nav-main">
        <Link to="/send" className="nav-primary">📤 Отправить</Link>
        <Link to="/receive" className="nav-primary">📥 Получить</Link>
        <Link to="/stake" className="nav-primary">📈 Стейкинг</Link>
      </nav>
      <nav className="nav nav-secondary">
        <Link to="/create-node">Создать узел</Link>
        <Link to="/help">Помощь</Link>
        <Link to="/settings">Настройки</Link>
      </nav>
      <section className="section">
        <h3>Последние переводы</h3>
        {txs.length === 0 ? (
          <p className="muted">Здесь появятся ваши переводы</p>
        ) : (
          <ul className="tx-list">
            {txs.map((tx) => (
              <li key={tx.id} className="tx-item">
                <span className="tx-amount">{(tx.amount / MICRO).toFixed(2)} ELENA</span>
                <span className="tx-time">{formatTimeAgo(tx.timestamp)}</span>
                <span className="tx-from-to" title="От кого → кому">
                  {shortHex(tx.from)} → {shortHex(tx.to)}
                </span>
              </li>
            ))}
          </ul>
        )}
      </section>
    </div>
  );
}
