//! API Gateway: REST + WebSocket для доступа к узлам «Елена».
//! Передаёт запросы к admin RPC узла (TCP line-based).

use axum::{
    extract::{Query, State, WebSocketUpgrade},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;

const DEFAULT_NODE_ADMIN: &str = "127.0.0.1:9190";

#[derive(Clone)]
struct AppState {
    node_admin: String,
    /// Если задан, для send/stake требуется заголовок X-API-Key или Authorization: Bearer <key>
    gateway_api_key: Option<String>,
}

fn check_api_key(state: &AppState, headers: &HeaderMap) -> bool {
    let Some(expected) = &state.gateway_api_key else { return true };
    let key = headers
        .get("X-API-Key")
        .and_then(|v| v.to_str().ok())
        .or_else(|| {
            headers
                .get("Authorization")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.strip_prefix("Bearer "))
        });
    key.map(|k| k.trim() == expected.as_str()).unwrap_or(false)
}

#[derive(Deserialize)]
struct SendBody {
    to: String,
    amount: u64,
}

#[derive(Deserialize)]
struct StakeBody {
    amount: f64,
}

#[derive(Serialize)]
struct StatusResponse {
    status: String,
    node: String,
}

async fn node_rpc(node_admin: &str, command: &str) -> Result<String, String> {
    let mut stream = TcpStream::connect(node_admin)
        .await
        .map_err(|e| format!("connect: {}", e))?;
    stream
        .write_all(command.as_bytes())
        .await
        .map_err(|e| format!("write: {}", e))?;
    stream.flush().await.map_err(|e| format!("flush: {}", e))?;
    let mut buf = String::new();
    stream
        .read_to_string(&mut buf)
        .await
        .map_err(|e| format!("read: {}", e))?;
    Ok(buf.trim().to_string())
}

async fn api_status(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match node_rpc(&state.node_admin, "stats\n").await {
        Ok(_) => (StatusCode::OK, Json(StatusResponse {
            status: "ok".to_string(),
            node: state.node_admin.clone(),
        })),
        Err(e) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(StatusResponse {
                status: format!("error: {}", e),
                node: state.node_admin.clone(),
            }),
        ),
    }
}

async fn api_network_stats(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match node_rpc(&state.node_admin, "stats\n").await {
        Ok(body) => {
            let v: serde_json::Value = serde_json::from_str(&body).unwrap_or(serde_json::json!({ "raw": body }));
            (StatusCode::OK, Json(v))
        }
        Err(e) => (
            StatusCode::BAD_GATEWAY,
            Json(serde_json::json!({ "error": e })),
        ),
    }
}

async fn api_network_parameters(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match node_rpc(&state.node_admin, "params\n").await {
        Ok(body) => {
            let v: serde_json::Value = serde_json::from_str(body.trim()).unwrap_or(serde_json::json!({ "error": "invalid json" }));
            (StatusCode::OK, Json(v))
        }
        Err(e) => (
            StatusCode::BAD_GATEWAY,
            Json(serde_json::json!({ "error": e })),
        ),
    }
}

async fn api_wallet_pubkey(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match node_rpc(&state.node_admin, "pubkey\n").await {
        Ok(pubkey) => (StatusCode::OK, Json(serde_json::json!({ "pubkey": pubkey }))),
        Err(e) => (
            StatusCode::BAD_GATEWAY,
            Json(serde_json::json!({ "error": e })),
        ),
    }
}

async fn api_transactions_recent(
    State(state): State<Arc<AppState>>,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let limit = params
        .get("limit")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(20)
        .min(100);
    let cmd = format!("recent_txs {}\n", limit);
    match node_rpc(&state.node_admin, &cmd).await {
        Ok(body) => {
            let v: serde_json::Value = serde_json::from_str(&body).unwrap_or(serde_json::json!([]));
            (StatusCode::OK, Json(v))
        }
        Err(e) => (
            StatusCode::BAD_GATEWAY,
            Json(serde_json::json!({ "error": e })),
        ),
    }
}

async fn api_transaction_send(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<SendBody>,
) -> impl IntoResponse {
    if !check_api_key(&state, &headers) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "API key required (X-API-Key or Authorization: Bearer)" })),
        );
    }
    let cmd = format!("send {} {}\n", body.to.trim(), body.amount);
    match node_rpc(&state.node_admin, &cmd).await {
        Ok(resp) => {
            if resp.starts_with("ok ") {
                let tx_id = resp.strip_prefix("ok ").unwrap_or("").to_string();
                (StatusCode::OK, Json(serde_json::json!({ "tx_id": tx_id })))
            } else {
                (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({ "error": resp })),
                )
            }
        }
        Err(e) => (
            StatusCode::BAD_GATEWAY,
            Json(serde_json::json!({ "error": e })),
        ),
    }
}

async fn api_wallet_stake(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<StakeBody>,
) -> impl IntoResponse {
    if !check_api_key(&state, &headers) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "API key required (X-API-Key or Authorization: Bearer)" })),
        );
    }
    let cmd = format!("stake {}\n", body.amount);
    match node_rpc(&state.node_admin, &cmd).await {
        Ok(resp) => {
            if resp == "ok" {
                (StatusCode::OK, Json(serde_json::json!({ "ok": true })))
            } else {
                (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({ "error": resp })),
                )
            }
        }
        Err(e) => (
            StatusCode::BAD_GATEWAY,
            Json(serde_json::json!({ "error": e })),
        ),
    }
}

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<Arc<AppState>>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| async move {
        let _ = socket;
        let _ = state;
    })
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env().add_directive("info".parse().unwrap()))
        .init();

    let node_admin = std::env::var("ELENA_NODE_ADMIN").unwrap_or_else(|_| DEFAULT_NODE_ADMIN.to_string());
    let gateway_api_key = std::env::var("GATEWAY_API_KEY").ok().filter(|s| !s.trim().is_empty());
    if gateway_api_key.is_some() {
        info!("Gateway API key is set; send/stake require X-API-Key or Authorization: Bearer");
    }
    let state = Arc::new(AppState {
        node_admin: node_admin.clone(),
        gateway_api_key,
    });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/api/v1/status", get(api_status))
        .route("/api/v1/network/stats", get(api_network_stats))
        .route("/api/v1/network/parameters", get(api_network_parameters))
        .route("/api/v1/wallet/pubkey", get(api_wallet_pubkey))
        .route("/api/v1/transactions/recent", get(api_transactions_recent))
        .route("/api/v1/transaction/send", post(api_transaction_send))
        .route("/api/v1/wallet/stake", post(api_wallet_stake))
        .route("/api/v1/ws", get(ws_handler))
        .layer(cors)
        .with_state(state);

    let addr = std::env::var("GATEWAY_LISTEN").unwrap_or_else(|_| "0.0.0.0:9180".to_string());
    info!("Gateway listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.expect("bind");
    axum::serve(listener, app).await.expect("serve");
}
