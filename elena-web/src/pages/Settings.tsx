import { useState, useEffect } from 'react';
import { Link } from 'react-router-dom';
import { getStoredApiUrl, setApiUrl, getApiKey, setApiKey } from '../services/api';

export function Settings() {
  const [apiUrl, setApiUrlState] = useState('');
  const [key, setKey] = useState('');
  const [saved, setSaved] = useState(false);

  useEffect(() => {
    setApiUrlState(getStoredApiUrl() ?? '');
    setKey(getApiKey() ?? '');
  }, []);

  const handleSave = () => {
    setApiUrl(apiUrl.trim() || null);
    setApiKey(key || null);
    setSaved(true);
    setTimeout(() => setSaved(false), 2000);
  };

  const handleClear = () => {
    setApiKey(null);
    setKey('');
  };

  return (
    <div className="page">
      <header className="header">
        <Link to="/">← Назад</Link>
        <h1>Настройки</h1>
      </header>
      <div className="card form">
        <h3>URL API (Gateway)</h3>
        <p className="muted">
          Адрес вашего gateway. Оставьте пустым для стандартного. Если вы создали свой узел — вставьте его URL (например, <code>https://elena-node-gateway-xxx.onrender.com</code>).
        </p>
        <label>
          URL API:
          <input
            type="url"
            value={apiUrl}
            onChange={(e) => setApiUrlState(e.target.value)}
            placeholder="http://localhost:9180"
          />
        </label>
      </div>
      <div className="card form">
        <h3>Дополнительно: API-ключ</h3>
        <p className="muted">
          Нужен только если владелец сайта включил защиту. Обычно поле можно не заполнять. Ключ выдаёт владелец сайта; он хранится только в этой вкладке браузера.
        </p>
        <label>
          API-ключ (если попросили):
          <input
            type="password"
            value={key}
            onChange={(e) => setKey(e.target.value)}
            placeholder="оставьте пустым, если gateway без ключа"
          />
        </label>
        <div className="form-actions">
          <button type="button" onClick={handleSave}>
            Сохранить
          </button>
          <button type="button" onClick={handleClear} className="secondary">
            Очистить ключ
          </button>
        </div>
        {saved && <p className="success">Сохранено.</p>}
      </div>
      <div className="card card-safety">
        <h3>Безопасность и резервная копия</h3>
        <p className="muted">
          Кошелёк хранится на узле сети. Если это ваш узел — сделайте резервную копию файла кошелька (.key) с сервера и храните её в безопасном месте. Кто владеет файлом — владеет средствами. Подробнее в разделе <Link to="/help">Помощь</Link>.
        </p>
      </div>
    </div>
  );
}
