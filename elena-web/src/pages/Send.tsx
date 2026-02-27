import { useState } from 'react';
import { Link } from 'react-router-dom';
import { sendTransaction } from '../services/api';

const MICRO = 1_000_000;

/** Парсит сумму: число в ELENA (допускает 0.5, 1, 10) → микро-ELENA */
function parseAmountElena(value: string): number | null {
  const v = value.replace(',', '.').trim();
  if (!v) return null;
  const n = parseFloat(v);
  if (Number.isNaN(n) || n <= 0) return null;
  return Math.floor(n * MICRO);
}

export function Send() {
  const [to, setTo] = useState('');
  const [amount, setAmount] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [txId, setTxId] = useState<string | null>(null);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);
    setTxId(null);
    const amtMicro = parseAmountElena(amount);
    if (!to.trim()) {
      setError('Введите адрес получателя');
      return;
    }
    if (amtMicro === null) {
      setError('Введите сумму в ELENA (например 1 или 0.5)');
      return;
    }
    setLoading(true);
    try {
      const r = await sendTransaction(to.trim(), amtMicro);
      setTxId(r.tx_id);
      setTo('');
      setAmount('');
    } catch (err) {
      const msg = err instanceof Error ? err.message : 'Ошибка отправки';
      setError(msg === 'API_KEY_REQUIRED' ? 'Требуется API-ключ. Введите его в Настройках.' : msg);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="page">
      <header className="header">
        <Link to="/">← Назад</Link>
        <h1>Отправить ELENA</h1>
      </header>
      <form onSubmit={handleSubmit} className="card form">
        <label>
          Адрес получателя
          <input
            type="text"
            value={to}
            onChange={(e) => setTo(e.target.value)}
            placeholder="Вставьте адрес кошелька получателя"
          />
        </label>
        <label>
          Сумма (ELENA)
          <input
            type="text"
            inputMode="decimal"
            value={amount}
            onChange={(e) => setAmount(e.target.value)}
            placeholder="Например: 1 или 0.5"
          />
        </label>
        {error && (
          <p className="error">
            {error}
            {error.includes('API-ключ') && (
              <> <Link to="/settings">Ввести ключ в настройках</Link></>
            )}
          </p>
        )}
        {txId && <p className="success">Готово. Платёж отправлен.</p>}
        <button type="submit" disabled={loading}>
          {loading ? 'Отправка…' : 'Отправить'}
        </button>
      </form>
    </div>
  );
}
