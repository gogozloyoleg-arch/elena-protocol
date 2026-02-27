import { Routes, Route, Link } from 'react-router-dom';
import { Overview } from './pages/Overview';
import { Nodes } from './pages/Nodes';
import { Parameters } from './pages/Parameters';
import './App.css';

function App() {
  return (
    <div className="app">
      <nav className="nav">
        <Link to="/">Обзор</Link>
        <Link to="/nodes">Узлы</Link>
        <Link to="/parameters">Параметры</Link>
      </nav>
      <main>
        <Routes>
          <Route path="/" element={<Overview />} />
          <Route path="/nodes" element={<Nodes />} />
          <Route path="/parameters" element={<Parameters />} />
        </Routes>
      </main>
    </div>
  );
}

export default App;
