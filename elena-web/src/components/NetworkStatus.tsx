import { useState, useEffect } from 'react';
import { getStatus } from '../services/api';

export function NetworkStatus() {
  const [status, setStatus] = useState<string>('—');
  const [node, setNode] = useState<string>('');

  useEffect(() => {
    getStatus()
      .then((r) => {
        setStatus(r.status === 'ok' ? 'live' : r.status);
        setNode(r.node);
      })
      .catch(() => setStatus('offline'));
  }, []);

  const label =
    status === 'live'
      ? 'Сеть работает'
      : status === 'offline'
        ? 'Нет связи'
        : `Ошибка: ${status}`;
  return (
    <span className={`network-status ${status}`} title={node ? `Подключение к сети` : undefined}>
      {status === 'live' ? '🟢 ' : status === 'offline' ? '🔴 ' : '⚠️ '}
      {label}
    </span>
  );
}
