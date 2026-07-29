//! REST + WebSocket remote API for energy-sims.

mod state;

use std::net::SocketAddr;
use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use clap::Parser;
use energy_sim_runtime::{AdvanceReport, Command, Session, Snapshot};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use state::{AppState, SessionStore};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::{info, warn};
use uuid::Uuid;

#[derive(Parser, Debug)]
#[command(name = "energy-sim-server", version, about = "Energy-sim REST + WebSocket API")]
struct Args {
    /// Listen address (default 127.0.0.1:8787).
    #[arg(long, default_value = "127.0.0.1:8787")]
    listen: SocketAddr,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "energy_sim_server=info,tower_http=info".into()),
        )
        .init();

    let args = Args::parse();
    let state = AppState {
        sessions: Arc::new(tokio::sync::Mutex::new(SessionStore::default())),
    };

    let app = Router::new()
        .route("/health", get(health))
        .route("/v1/sessions", post(create_session))
        .route("/v1/sessions/{id}", get(get_session))
        .route("/v1/sessions/{id}/start", post(start_session))
        .route("/v1/sessions/{id}/stop", post(stop_session))
        .route("/v1/sessions/{id}/advance", post(advance_session))
        .route("/v1/sessions/{id}/tick", post(tick_session))
        .route("/v1/sessions/{id}/commands", post(apply_commands))
        .route("/v1/sessions/{id}/history", get(history))
        .route("/v1/sessions/{id}/checkpoint", post(checkpoint))
        .route("/v1/sessions/{id}/live", get(live_ws))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    info!("listening on http://{}", args.listen);
    let listener = tokio::net::TcpListener::bind(args.listen).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "ok": true,
        "engine": energy_sim_runtime::engine_banner(),
    }))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CreateSessionResponse {
    session_id: String,
    snapshot: Snapshot,
}

async fn create_session(
    State(state): State<AppState>,
    body: axum::body::Bytes,
) -> Result<Json<CreateSessionResponse>, ApiError> {
    let text = std::str::from_utf8(&body).map_err(|e| ApiError::bad_request(e.to_string()))?;
    let session = Session::from_json(text).map_err(ApiError::from_runtime)?;
    let id = Uuid::new_v4().to_string();
    let snapshot = session.snapshot();
    state.sessions.lock().await.insert(id.clone(), session);
    Ok(Json(CreateSessionResponse {
        session_id: id,
        snapshot,
    }))
}

async fn get_session(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Snapshot>, ApiError> {
    let store = state.sessions.lock().await;
    let session = store.get(&id).ok_or_else(ApiError::not_found)?;
    Ok(Json(session.snapshot()))
}

async fn start_session(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Snapshot>, ApiError> {
    let mut store = state.sessions.lock().await;
    let session = store.get_mut(&id).ok_or_else(ApiError::not_found)?;
    session.start().map_err(ApiError::from_runtime)?;
    Ok(Json(session.snapshot()))
}

async fn stop_session(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Snapshot>, ApiError> {
    let mut store = state.sessions.lock().await;
    let session = store.get_mut(&id).ok_or_else(ApiError::not_found)?;
    session.stop().map_err(ApiError::from_runtime)?;
    Ok(Json(session.snapshot()))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AdvanceBody {
    duration_secs: f64,
    #[serde(default)]
    commands: Vec<Command>,
}

async fn advance_session(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<AdvanceBody>,
) -> Result<Json<AdvanceReport>, ApiError> {
    let mut store = state.sessions.lock().await;
    let session = store.get_mut(&id).ok_or_else(ApiError::not_found)?;
    for cmd in body.commands {
        session.apply(cmd).map_err(ApiError::from_runtime)?;
    }
    let report = session
        .advance_secs(body.duration_secs)
        .map_err(ApiError::from_runtime)?;
    Ok(Json(report))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TickBody {
    dt_secs: f64,
}

async fn tick_session(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<TickBody>,
) -> Result<Json<Snapshot>, ApiError> {
    let mut store = state.sessions.lock().await;
    let session = store.get_mut(&id).ok_or_else(ApiError::not_found)?;
    let snap = session
        .tick_secs(body.dt_secs)
        .map_err(ApiError::from_runtime)?;
    Ok(Json(snap))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CommandsBody {
    commands: Vec<Command>,
}

async fn apply_commands(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<CommandsBody>,
) -> Result<Json<Snapshot>, ApiError> {
    let mut store = state.sessions.lock().await;
    let session = store.get_mut(&id).ok_or_else(ApiError::not_found)?;
    for cmd in body.commands {
        session.apply(cmd).map_err(ApiError::from_runtime)?;
    }
    Ok(Json(session.snapshot()))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct HistoryQuery {
    from_secs: Option<f64>,
    to_secs: Option<f64>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct HistoryResponse {
    events: Vec<energy_sim_runtime::Event>,
    samples: Vec<energy_sim_runtime::Sample>,
}

async fn history(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(q): Query<HistoryQuery>,
) -> Result<Json<HistoryResponse>, ApiError> {
    let store = state.sessions.lock().await;
    let session = store.get(&id).ok_or_else(ApiError::not_found)?;
    let from = q.from_secs.unwrap_or(0.0);
    let to = q.to_secs.unwrap_or(f64::MAX);
    let events = session
        .events()
        .iter()
        .filter(|e| e.sim_time_s >= from && e.sim_time_s <= to)
        .cloned()
        .collect();
    let samples = session
        .samples()
        .iter()
        .filter(|s| s.sim_time_s >= from && s.sim_time_s <= to)
        .cloned()
        .collect();
    Ok(Json(HistoryResponse { events, samples }))
}

async fn checkpoint(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let store = state.sessions.lock().await;
    let session = store.get(&id).ok_or_else(ApiError::not_found)?;
    let value = session.checkpoint_value().map_err(ApiError::from_runtime)?;
    Ok(Json(value))
}

async fn live_ws(
    State(state): State<AppState>,
    Path(id): Path<String>,
    ws: WebSocketUpgrade,
) -> Result<Response, ApiError> {
    {
        let store = state.sessions.lock().await;
        if !store.contains_key(&id) {
            return Err(ApiError::not_found());
        }
    }
    Ok(ws.on_upgrade(move |socket| handle_live(socket, state, id)))
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum LiveClientMessage {
    Tick { dt_secs: f64 },
    Advance { duration_secs: f64 },
    Command { command: Command },
    Ping,
}

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum LiveServerMessage {
    Snapshot { snapshot: Snapshot },
    Error { message: String },
    Pong,
}

async fn handle_live(socket: WebSocket, state: AppState, id: String) {
    let (mut sender, mut receiver) = socket.split();

    // Initial snapshot.
    if let Some(snap) = {
        let store = state.sessions.lock().await;
        store.get(&id).map(|s| s.snapshot())
    } {
        let msg = LiveServerMessage::Snapshot { snapshot: snap };
        if send_json(&mut sender, &msg).await.is_err() {
            return;
        }
    } else {
        let _ = send_json(
            &mut sender,
            &LiveServerMessage::Error {
                message: "session not found".into(),
            },
        )
        .await;
        return;
    }

    while let Some(Ok(msg)) = receiver.next().await {
        match msg {
            Message::Text(text) => {
                let client: LiveClientMessage = match serde_json::from_str(&text) {
                    Ok(m) => m,
                    Err(e) => {
                        let _ = send_json(
                            &mut sender,
                            &LiveServerMessage::Error {
                                message: format!("bad client message: {e}"),
                            },
                        )
                        .await;
                        continue;
                    }
                };
                let result = {
                    let mut store = state.sessions.lock().await;
                    match store.get_mut(&id) {
                        None => Err("session not found".to_string()),
                        Some(session) => match client {
                            LiveClientMessage::Ping => Ok(LiveServerMessage::Pong),
                            LiveClientMessage::Tick { dt_secs } => session
                                .tick_secs(dt_secs)
                                .map(|snapshot| LiveServerMessage::Snapshot { snapshot })
                                .map_err(|e| e.to_string()),
                            LiveClientMessage::Advance { duration_secs } => session
                                .advance_secs(duration_secs)
                                .map(|r| LiveServerMessage::Snapshot {
                                    snapshot: r.snapshot,
                                })
                                .map_err(|e| e.to_string()),
                            LiveClientMessage::Command { command } => session
                                .apply(command)
                                .map(|_| LiveServerMessage::Snapshot {
                                    snapshot: session.snapshot(),
                                })
                                .map_err(|e| e.to_string()),
                        },
                    }
                };
                match result {
                    Ok(msg) => {
                        if send_json(&mut sender, &msg).await.is_err() {
                            break;
                        }
                    }
                    Err(message) => {
                        warn!(%id, %message, "live command failed");
                        let _ = send_json(&mut sender, &LiveServerMessage::Error { message }).await;
                    }
                }
            }
            Message::Close(_) => break,
            Message::Ping(p) => {
                let _ = sender.send(Message::Pong(p)).await;
            }
            _ => {}
        }
    }
}

async fn send_json<S: Serialize>(
    sender: &mut futures_util::stream::SplitSink<WebSocket, Message>,
    value: &S,
) -> Result<(), ()> {
    let text = serde_json::to_string(value).map_err(|_| ())?;
    sender.send(Message::Text(text.into())).await.map_err(|_| ())
}

// --- API errors ---

struct ApiError {
    status: StatusCode,
    message: String,
}

impl ApiError {
    fn bad_request(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            message: message.into(),
        }
    }

    fn not_found() -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            message: "session not found".into(),
        }
    }

    fn from_runtime(err: energy_sim_runtime::RuntimeError) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            message: err.to_string(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = Json(serde_json::json!({ "error": self.message }));
        (self.status, body).into_response()
    }
}
