"""
Простой веб-дашборд на FastAPI: граф, метрики, анимация распространения, WebSocket.
"""

import json
from pathlib import Path
from typing import Optional, Dict, Any

from fastapi import FastAPI, WebSocket, WebSocketDisconnect
from fastapi.responses import HTMLResponse, FileResponse

# Глобальное состояние для дашборда (заполняется из main при --viz)
dashboard_state: Dict[str, Any] = {
    "graph": None,
    "metrics": None,
    "runner": None,
}

_DASHBOARD_HTML_PATH = Path(__file__).resolve().parent / "dashboard_page.html"


def create_app() -> FastAPI:
    app = FastAPI(title="Елена — симулятор сети")

    @app.get("/")
    async def root():
        """Главная страница: интерактивный граф и анимация расхождения транзакций."""
        if _DASHBOARD_HTML_PATH.exists():
            return FileResponse(_DASHBOARD_HTML_PATH, media_type="text/html")
        return HTMLResponse(
            "<h1>Елена</h1><p><a href='/graph'>Граф (JSON)</a> | <a href='/metrics'>Метрики</a></p>"
        )

    @app.get("/graph")
    def get_graph():
        """Возвращает текущее состояние графа в JSON."""
        state = dashboard_state.get("graph") or dashboard_state.get("runner")
        if state is None:
            return {"nodes": [], "edges": [], "transactions_count": 0, "alerts_count": 0}
        if hasattr(state, "graph"):
            g = state.graph
        else:
            g = state
        nodes = [
            {"id": nid, "reputation": round(n.reputation, 2), "is_evil": getattr(n, "is_evil", False)}
            for nid, n in g.nodes.items()
        ]
        edges = []
        seen = set()
        for nid, node in g.nodes.items():
            for peer in node.peers:
                key = tuple(sorted([nid, peer.id]))
                if key not in seen:
                    seen.add(key)
                    edges.append({"source": nid, "target": peer.id})
        return {
            "nodes": nodes,
            "edges": edges,
            "transactions_count": len(g.transactions),
            "alerts_count": len(g.alerts),
        }

    @app.get("/metrics")
    def get_metrics():
        """Возвращает метрики симуляции."""
        m = dashboard_state.get("metrics")
        if m is None:
            runner = dashboard_state.get("runner")
            m = getattr(runner, "metrics", None)
        if m is None:
            return {}
        return m.get_summary()

    @app.websocket("/ws")
    async def websocket_endpoint(websocket: WebSocket):
        await websocket.accept()
        try:
            while True:
                data = await websocket.receive_text()
                if data == "ping":
                    metrics = dashboard_state.get("metrics") or (dashboard_state.get("runner") and dashboard_state.get("runner").metrics)
                    payload = metrics.get_summary() if metrics else {}
                    await websocket.send_json({"type": "metrics", "data": payload})
                else:
                    await websocket.send_json({"type": "pong"})
        except WebSocketDisconnect:
            pass

    return app


def set_dashboard_state(runner=None, graph=None, metrics=None):
    """Устанавливает состояние для дашборда."""
    if runner:
        dashboard_state["runner"] = runner
        dashboard_state["graph"] = getattr(runner, "graph", None)
        dashboard_state["metrics"] = getattr(runner, "metrics", None)
    if graph is not None:
        dashboard_state["graph"] = graph
    if metrics is not None:
        dashboard_state["metrics"] = metrics
