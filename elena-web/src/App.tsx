import { Routes, Route } from 'react-router-dom';
import { Dashboard } from './pages/Dashboard';
import { Send } from './pages/Send';
import { Receive } from './pages/Receive';
import { Stake } from './pages/Stake';
import { Settings } from './pages/Settings';
import { Help } from './pages/Help';
import { CreateNode } from './pages/CreateNode';
import './App.css';

function App() {
  return (
    <Routes>
      <Route path="/" element={<Dashboard />} />
      <Route path="/send" element={<Send />} />
      <Route path="/receive" element={<Receive />} />
      <Route path="/stake" element={<Stake />} />
      <Route path="/settings" element={<Settings />} />
      <Route path="/help" element={<Help />} />
      <Route path="/create-node" element={<CreateNode />} />
    </Routes>
  );
}

export default App;
