import type { NetworkStats } from '../services/api';

const MICRO = 1_000_000;

export function WalletCard({ stats }: { stats: NetworkStats | null }) {
  if (!stats) return <div className="card">Загрузка…</div>;
  const balanceElena = (stats.balance / MICRO).toFixed(2);
  const rep = Object.values(stats.reputation)[0] ?? 0;
  const stars = Math.min(5, Math.round(rep * 5));

  return (
    <div className="card wallet-card">
      <h3>Ваш баланс</h3>
      <p className="balance">{balanceElena} ELENA</p>
      <p className="meta balance-hint">Средства в сети ELENA. Ими можно расплачиваться и участвовать в стейкинге.</p>
      <p className="reputation">
        Доверие в сети: {'⭐'.repeat(stars)} {rep.toFixed(1)}
      </p>
      <p className="meta meta-secondary">
        Переводов: {stats.transactions}
        {stats.alerts > 0 && ` · Уведомлений: ${stats.alerts}`}
      </p>
    </div>
  );
}
