import { useState, useEffect } from 'react';
import { Link } from 'react-router-dom';
import { getPubkey } from '../services/api';

export function Receive() {
  const [pubkey, setPubkey] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    getPubkey()
      .then((r) => setPubkey(r.pubkey))
      .catch((e) => setError(e.message));
  }, []);

  const copy = () => {
    if (pubkey) navigator.clipboard.writeText(pubkey);
  };

  return (
    <div className="page">
      <header className="header">
        <Link to="/">← Назад</Link>
        <h1>Получить</h1>
      </header>
      <div className="card">
        <h3>Ваш адрес для приёма ELENA</h3>
        <p className="muted">Скопируйте адрес и отправьте тому, кто будет переводить вам средства.</p>
        {error && <p className="error">{error}</p>}
        {pubkey && (
          <>
            <p className="pubkey" title={pubkey}>
              {pubkey.slice(0, 24)}…{pubkey.slice(-16)}
            </p>
            <button onClick={copy}>Копировать адрес</button>
          </>
        )}
      </div>
    </div>
  );
}
