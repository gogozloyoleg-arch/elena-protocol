import { useState, useEffect, useCallback } from 'react';
import { getGatewayUrls, getNetworkStatsFrom, getStatusFrom } from '../services/api';
import type { NetworkStats } from '../services/api';

const STORAGE_KEY = 'elena_owner_gateway_urls';

function loadExtraUrls(): string[] {
  try {
    const s = localStorage.getItem(STORAGE_KEY);
    if (s) return JSON.parse(s);
  } catch {}
  return [];
}

function saveExtraUrls(urls: string[]) {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(urls));
  } catch {}
}

type NodeState =
  | { status: 'loading'; url: string }
  | { status: 'ok'; url: string; label: string; stats: NetworkStats; statusText: string }
  | { status: 'error'; url: string; label: string; error: string };

function shortPeerId(pid: string, n = 12): string {
  if (pid.length <= n) return pid;
  return pid.slice(0, n) + '…';
}

export function Nodes() {
  const [extraUrls, setExtraUrls] = useState<string[]>(loadExtraUrls);
  const [nodes, setNodes] = useState<NodeState[]>([]);
  const [newUrl, setNewUrl] = useState('');

  const allUrls = [...getGatewayUrls(), ...extraUrls];

  const refresh = useCallback(async () => {
    const urls = [...getGatewayUrls(), ...extraUrls];
    if (urls.length === 0) {
      setNodes([{ status: 'error', url: '', label: '—', error: 'Нет адресов gateway. Добавьте URL в настройках или задайте VITE_GATEWAY_URLS.' }]);
      return;
    }
    setNodes(urls.map((url) => ({ status: 'loading' as const, url })));
    const results: NodeState[] = await Promise.all(
      urls.map(async (url): Promise<NodeState> => {
        let label = url;
        try {
          const u = new URL(url.startsWith('http') ? url : `http://${url}`);
          label = u.hostname + (u.port && u.port !== '80' ? `:${u.port}` : '');
        } catch {}
        try {
          const [statsRes, statusRes] = await Promise.all([
            getNetworkStatsFrom(url),
            getStatusFrom(url).catch(() => ({ status: 'error', node: url })),
          ]);
          const statusText = statusRes.status === 'ok' ? 'работает' : statusRes.status;
          return { status: 'ok', url, label, stats: statsRes, statusText };
        } catch (e) {
          return { status: 'error', url, label, error: e instanceof Error ? e.message : String(e) };
        }
      })
    );
    setNodes(results);
  }, [allUrls.length]);

  useEffect(() => {
    refresh();
  }, [refresh]);

  const addUrl = () => {
    const u = newUrl.trim();
    if (!u) return;
    const norm = u.replace(/\/$/, '');
    if (allUrls.some((x) => x.replace(/\/$/, '') === norm)) return;
    const next = [...extraUrls, norm];
    setExtraUrls(next);
    saveExtraUrls(next);
    setNewUrl('');
  };

  const removeUrl = (url: string) => {
    const next = extraUrls.filter((x) => x.replace(/\/$/, '') !== url.replace(/\/$/, ''));
    setExtraUrls(next);
    saveExtraUrls(next);
  };

  return (
    <div className="page">
      <h1>Узлы</h1>
      <p className="muted">Мониторинг узлов через подключённые gateway. Каждый gateway привязан к одному узлу.</p>

      <div className="card form-card">
        <h3>Добавить узел (URL gateway)</h3>
        <div className="form-row">
          <input
            type="text"
            value={newUrl}
            onChange={(e) => setNewUrl(e.target.value)}
            placeholder="https://example.com:9180"
          />
          <button type="button" onClick={addUrl}>Добавить</button>
        </div>
        {extraUrls.length > 0 && (
          <ul className="url-list">
            {extraUrls.map((u) => (
              <li key={u}>
                <code>{u}</code>
                <button type="button" className="btn-remove" onClick={() => removeUrl(u)}>Удалить</button>
              </li>
            ))}
          </ul>
        )}
      </div>

      <section className="nodes-section">
        <h2>Подключённые узлы</h2>
        <button type="button" className="btn-refresh" onClick={refresh}>Обновить</button>
        {nodes.length === 0 && <p className="muted">Загрузка…</p>}
        {nodes.map((node, i) => {
          if (node.status === 'loading') {
            return (
              <div key={node.url} className="card node-card node-loading">
                <h3>Узел {i + 1}</h3>
                <p>Загрузка…</p>
              </div>
            );
          }
          if (node.status === 'error') {
            return (
              <div key={node.url} className="card node-card node-error">
                <h3>{node.label}</h3>
                <p><strong>Ошибка:</strong> {node.error}</p>
                {extraUrls.includes(node.url) && (
                  <button type="button" className="btn-remove" onClick={() => removeUrl(node.url)}>Удалить из списка</button>
                )}
              </div>
            );
          }
          const rep = Object.entries(node.stats.reputation)[0];
          const stars = rep ? Math.min(5, Math.round(rep[1] * 5)) : 0;
          const balanceElena = (node.stats.balance / 1_000_000).toFixed(2);
          return (
            <div key={node.url} className="card node-card">
              <h3>{node.label}</h3>
              <p className="node-status"><span className={node.statusText === 'работает' ? 'status-ok' : 'status-warn'}>{node.statusText}</span></p>
              <p><strong>Peer ID:</strong> <code>{shortPeerId(node.stats.peer_id)}</code></p>
              <p><strong>Баланс:</strong> {balanceElena} ELENA</p>
              <p><strong>Репутация:</strong> {rep ? rep[1].toFixed(2) : '—'} {'⭐'.repeat(stars)}</p>
              <p><strong>Транзакций в графе:</strong> {node.stats.transactions}</p>
              {node.stats.alerts > 0 && <p><strong>Алертов:</strong> {node.stats.alerts}</p>}
              {extraUrls.includes(node.url) && (
                <button type="button" className="btn-remove" onClick={() => removeUrl(node.url)}>Удалить из списка</button>
              )}
            </div>
          );
        })}
      </section>

      <p className="muted">Чтобы мониторить несколько узлов, запустите для каждого свой gateway (и узел), затем добавьте сюда URL каждого gateway.</p>
    </div>
  );
}
