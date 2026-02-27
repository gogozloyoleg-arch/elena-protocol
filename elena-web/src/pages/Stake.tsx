import { useState } from 'react';
import { Link } from 'react-router-dom';
import { stake as apiStake } from '../services/api';

const apiKeyError = (s: string) => s.includes('API-ключ');

export function Stake() {
  const [amount, setAmount] = useState('0.3');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [ok, setOk] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);
    setOk(false);
    const v = parseFloat(amount);
    if (isNaN(v) || v < 0 || v > 0.5) {
      setError('Доля должна быть от 0 до 0.5');
      return;
    }
    setLoading(true);
    try {
      await apiStake(v);
      setOk(true);
    } catch (err) {
      const msg = err instanceof Error ? err.message : 'Ошибка';
      setError(msg === 'API_KEY_REQUIRED' ? 'Требуется API-ключ для стейкинга.' : msg);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="page">
      <header className="header">
        <Link to="/">← Назад</Link>
        <h1>Стейкинг</h1>
      </header>
      <div className="card">
        <p>Укажите долю репутации для стейкинга (от 0 до 0.5). Чем больше — тем выше ваша доля в вознаграждениях сети.</p>
        <form onSubmit={handleSubmit} className="form">
          <label>
            Доля (0 — 0.5):
            <input
              type="number"
              step="0.01"
              min="0"
              max="0.5"
              value={amount}
              onChange={(e) => setAmount(e.target.value)}
            />
          </label>
          {error && (
            <p className="error">
              {error}
              {apiKeyError(error) && (
                <> <Link to="/settings">Ввести ключ в настройках</Link></>
              )}
            </p>
          )}
          {ok && <p className="success">Стейкинг установлен</p>}
          <button type="submit" disabled={loading}>
            {loading ? 'Сохранение…' : 'Установить'}
          </button>
        </form>
      </div>
    </div>
  );
}
