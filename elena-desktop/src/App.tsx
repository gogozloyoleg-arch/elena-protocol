import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';

const GATEWAY_URL = 'http://localhost:9180';

type NodeStatus = 'unknown' | 'running' | 'stopped' | 'starting' | 'error';

async function checkGateway(): Promise<boolean> {
  try {
    const r = await fetch(`${GATEWAY_URL}/api/v1/status`, { signal: AbortSignal.timeout(2000) });
    return r.ok;
  } catch {
    return false;
  }
}

function App() {
  const [dataDir, setDataDir] = useState<string>('');
  const [nodeStatus, setNodeStatus] = useState<NodeStatus>('unknown');
  const [message, setMessage] = useState<string>('');

  const refreshStatus = async () => {
    const ok = await checkGateway();
    setNodeStatus(ok ? 'running' : 'stopped');
  };

  useEffect(() => {
    invoke<string>('get_elena_data_dir')
      .then(setDataDir)
      .catch(() => setDataDir(''));
    refreshStatus();
  }, []);

  const handleStartNode = async () => {
    setMessage('');
    setNodeStatus('starting');
    try {
      await invoke('ensure_wallet');
      await invoke('start_node');
      for (let i = 0; i < 15; i++) {
        await new Promise((r) => setTimeout(r, 800));
        if (await checkGateway()) {
          setNodeStatus('running');
          setMessage('Узел запущен. Можно открыть кошелёк.');
          return;
        }
      }
      setNodeStatus('error');
      setMessage('Узел не ответил вовремя. Проверьте, что бинарники elena-core и elena-gateway лежат в resources/bin/');
    } catch (e) {
      setNodeStatus('stopped');
      setMessage(String(e));
    }
  };

  const handleStopNode = async () => {
    setMessage('');
    try {
      await invoke('stop_node');
      setNodeStatus('stopped');
      setMessage('Узел остановлен.');
    } catch (e) {
      setMessage(String(e));
    }
  };

  const handleOpenWallet = () => {
    window.open(GATEWAY_URL, '_blank');
  };

  return (
    <div style={{ padding: '1.5rem', fontFamily: 'system-ui', maxWidth: 420 }}>
      <h1 style={{ marginTop: 0 }}>ELENA Wallet</h1>
      <p style={{ color: '#666' }}>Ваш узел и кошелёк в один клик.</p>

      {dataDir && (
        <p style={{ fontSize: '0.85rem', color: '#888', wordBreak: 'break-all' }}>
          Данные: {dataDir}
        </p>
      )}

      <div style={{ marginTop: '1.5rem', display: 'flex', flexDirection: 'column', gap: '0.75rem' }}>
        {nodeStatus === 'running' && (
          <>
            <p style={{ color: 'green', fontWeight: 600 }}>Сеть работает</p>
            <button type="button" onClick={handleOpenWallet} style={{ padding: '0.75rem 1rem', fontSize: '1rem' }}>
              Открыть кошелёк
            </button>
            <button type="button" onClick={handleStopNode} style={{ padding: '0.5rem', background: '#ddd' }}>
              Остановить узел
            </button>
          </>
        )}
        {(nodeStatus === 'stopped' || nodeStatus === 'error') && (
          <>
            <button type="button" onClick={handleStartNode} style={{ padding: '0.75rem 1.25rem', fontSize: '1rem', background: '#2dd4bf', color: '#0c1222', border: 'none', borderRadius: 8, fontWeight: 600, cursor: 'pointer' }}>
              Запустить узел
            </button>
            <p style={{ fontSize: '0.9rem', color: '#666' }}>
              При первом запуске будет создан кошелёк и запущены узел и gateway. Затем откройте кошелёк в браузере.
            </p>
          </>
        )}
        {nodeStatus === 'starting' && (
          <p style={{ color: '#888' }}>Запуск узла…</p>
        )}
      </div>

      {message && (
        <p style={{ marginTop: '1rem', padding: '0.75rem', background: message.includes('ошибка') || nodeStatus === 'error' ? '#fee' : '#efe', borderRadius: 8, fontSize: '0.9rem' }}>
          {message}
        </p>
      )}
    </div>
  );
}

export default App;
