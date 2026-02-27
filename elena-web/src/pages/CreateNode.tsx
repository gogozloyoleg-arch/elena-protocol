import { useState } from 'react';
import { Link } from 'react-router-dom';

const RENDER_REPO = 'https://github.com/gogozloyoleg-arch/elena-protocol';
const DOCKER_CMD = `git clone --depth 1 ${RENDER_REPO}.git elena-node && cd elena-node && docker compose -f docker-compose.node.yml up -d`;

export function CreateNode() {
  const [copied, setCopied] = useState(false);

  const copyCmd = () => {
    navigator.clipboard.writeText(DOCKER_CMD);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div className="page">
      <header className="header">
        <Link to="/">← Назад</Link>
        <h1>Создать свой узел</h1>
      </header>

      <p className="muted create-intro">
        Свой узел = свой кошелёк. Ключи хранятся у вас, средства под вашим контролем.
      </p>

      <div className="card card-highlight">
        <h2>1. Через веб (Render)</h2>
        <p>Один клик — узел и gateway в облаке. Нужен аккаунт на Render (бесплатный план).</p>
        <a
          href={`https://render.com/deploy?repo=${encodeURIComponent(RENDER_REPO)}`}
          target="_blank"
          rel="noopener noreferrer"
          className="btn-deploy"
        >
          Deploy to Render
        </a>
        <p className="muted small">
          После деплоя Render даст URL вида <code>https://elena-node-gateway-xxx.onrender.com</code>.
          Скопируйте его и вставьте в <Link to="/settings">Настройки</Link> → URL API.
        </p>
      </div>

      <div className="card">
        <h2>2. Docker (свой сервер или компьютер)</h2>
        <p>Скопируйте команду и выполните в терминале. Через минуту gateway будет на порту 9180.</p>
        <div className="code-block">
          <code>{DOCKER_CMD}</code>
          <button type="button" className="btn-copy" onClick={copyCmd}>
            {copied ? 'Скопировано!' : 'Копировать'}
          </button>
        </div>
        <p className="muted small">
          Если у вас VPS с публичным IP — откройте порт 9180. URL: <code>http://ваш-ip:9180</code>.
        </p>
      </div>

      <div className="card">
        <h2>3. Подключить кошелёк</h2>
        <ol className="list list-simple">
          <li>Получите URL gateway (из Render или ваш сервер: порт 9180).</li>
          <li>Откройте <Link to="/settings">Настройки</Link>.</li>
          <li>Вставьте URL в поле «URL API» (например, <code>https://elena-node-gateway-xxx.onrender.com</code>).</li>
          <li>Сохраните. Готово — вы в своём кошельке.</li>
        </ol>
      </div>

      <div className="card muted">
        <h3>Резервная копия</h3>
        <p>
          Ключи кошелька хранятся на узле в файле <code>wallets/mynode.key</code>. Для Docker — в volume.
          Скопируйте его в безопасное место: кто владеет файлом — владеет средствами.
        </p>
      </div>
    </div>
  );
}
