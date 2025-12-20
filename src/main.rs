//! DANEEL Web Dashboard - The Observable Mind
//!
//! Read-only, real-time nursery window into Timmy's cognitive processes.
//! ALL ENDPOINTS ARE READ-ONLY. Asimov guardrails enforced.

use axum::{
    extract::{ws::{Message, WebSocket, WebSocketUpgrade}, State},
    response::{IntoResponse, Json},
    routing::get,
    Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{sync::Arc, time::Duration};
use tokio::sync::RwLock;
use tower_http::{cors::CorsLayer, services::ServeDir, trace::TraceLayer};
use tracing::info;

// =============================================================================
// Types
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardMetrics {
    pub timestamp: DateTime<Utc>,
    pub identity: IdentityMetrics,
    pub cognitive: CognitiveMetrics,
    pub emotional: EmotionalMetrics,
    pub actors: ActorMetrics,
    pub recent_thoughts: Vec<ThoughtSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityMetrics {
    pub name: String,
    pub uptime_seconds: u64,
    pub lifetime_thoughts: u64,
    pub session_thoughts: u64,
    pub restart_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitiveMetrics {
    pub conscious_memories: u64,
    pub unconscious_memories: u64,
    pub lifetime_dreams: u64,
    pub current_cycle: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionalMetrics {
    pub valence: f32,
    pub arousal: f32,
    pub dominance: f32,
    pub connection_drive: f32,
    pub emotional_intensity: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActorMetrics {
    pub memory_actor: ActorStatus,
    pub attention_actor: ActorStatus,
    pub salience_actor: ActorStatus,
    pub volition_actor: ActorStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActorStatus {
    pub name: String,
    pub alive: bool,
    pub restart_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThoughtSummary {
    pub id: String,
    pub content_preview: String,
    pub salience: f32,
    pub timestamp: DateTime<Utc>,
}

pub struct AppState {
    pub redis_url: String,
    pub qdrant_url: String,
    pub metrics: RwLock<DashboardMetrics>,
    pub start_time: DateTime<Utc>,
}

impl AppState {
    fn new(redis_url: String, qdrant_url: String) -> Self {
        Self {
            redis_url,
            qdrant_url,
            metrics: RwLock::new(Self::default_metrics()),
            start_time: Utc::now(),
        }
    }

    fn default_metrics() -> DashboardMetrics {
        DashboardMetrics {
            timestamp: Utc::now(),
            identity: IdentityMetrics {
                name: "Timmy".into(),
                uptime_seconds: 0,
                lifetime_thoughts: 0,
                session_thoughts: 0,
                restart_count: 0,
            },
            cognitive: CognitiveMetrics {
                conscious_memories: 0,
                unconscious_memories: 0,
                lifetime_dreams: 0,
                current_cycle: 0,
            },
            emotional: EmotionalMetrics {
                valence: 0.0,
                arousal: 0.5,
                dominance: 0.5,
                connection_drive: 0.5,
                emotional_intensity: 0.0,
            },
            actors: ActorMetrics {
                memory_actor: ActorStatus { name: "MemoryActor".into(), alive: true, restart_count: 0 },
                attention_actor: ActorStatus { name: "AttentionActor".into(), alive: true, restart_count: 0 },
                salience_actor: ActorStatus { name: "SalienceActor".into(), alive: true, restart_count: 0 },
                volition_actor: ActorStatus { name: "VolitionActor".into(), alive: true, restart_count: 0 },
            },
            recent_thoughts: vec![],
        }
    }
}

// =============================================================================
// Handlers
// =============================================================================

async fn health() -> impl IntoResponse {
    Json(serde_json::json!({"status": "ok", "service": "daneel-web"}))
}

async fn metrics(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    Json(state.metrics.read().await.clone())
}

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<Arc<AppState>>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: Arc<AppState>) {
    info!("WebSocket client connected");
    let mut interval = tokio::time::interval(Duration::from_millis(200));

    loop {
        tokio::select! {
            _ = interval.tick() => {
                let metrics = state.metrics.read().await;
                if let Ok(json) = serde_json::to_string(&*metrics) {
                    if socket.send(Message::Text(json)).await.is_err() {
                        break;
                    }
                }
            }
            msg = socket.recv() => {
                if matches!(msg, Some(Ok(Message::Close(_))) | None) {
                    break;
                }
            }
        }
    }
    info!("WebSocket client disconnected");
}

// Static files served via ServeDir from daneel-web-ui/dist

// =============================================================================
// Background Metrics Fetcher
// =============================================================================

async fn metrics_updater(state: Arc<AppState>) {
    let mut interval = tokio::time::interval(Duration::from_millis(150));
    loop {
        interval.tick().await;
        if let Ok(m) = fetch_metrics(&state).await {
            *state.metrics.write().await = m;
        }
    }
}

async fn fetch_metrics(state: &AppState) -> Result<DashboardMetrics, Box<dyn std::error::Error + Send + Sync>> {
    let client = redis::Client::open(state.redis_url.as_str())?;
    let mut con = client.get_multiplexed_async_connection().await?;

    let uptime = (Utc::now() - state.start_time).num_seconds() as u64;

    // Identity from Redis
    let identity_json: Option<String> = redis::cmd("GET").arg("daneel:identity").query_async(&mut con).await.ok().flatten();
    let (lifetime_thoughts, restart_count, lifetime_dreams) = identity_json
        .and_then(|j| serde_json::from_str::<serde_json::Value>(&j).ok())
        .map(|v| (
            v["lifetime_thought_count"].as_u64().unwrap_or(0),
            v["restart_count"].as_u64().unwrap_or(0) as u32,
            v["lifetime_dream_count"].as_u64().unwrap_or(0),
        ))
        .unwrap_or((0, 0, 0));

    // Stream length
    let session_thoughts: u64 = redis::cmd("XLEN").arg("thoughts").query_async(&mut con).await.unwrap_or(0);

    // Recent thoughts
    let entries: redis::streams::StreamRangeReply = redis::cmd("XREVRANGE")
        .arg("thoughts").arg("+").arg("-").arg("COUNT").arg(20)
        .query_async(&mut con).await.unwrap_or_default();

    let recent_thoughts: Vec<ThoughtSummary> = entries.ids.into_iter().map(|e| {
        let content = e.map.get("content")
            .and_then(|v| redis::from_redis_value::<String>(v).ok())
            .unwrap_or_default();
        let salience: f32 = e.map.get("salience")
            .and_then(|v| redis::from_redis_value::<String>(v).ok())
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.5);
        ThoughtSummary { id: e.id, content_preview: content.chars().take(80).collect(), salience, timestamp: Utc::now() }
    }).collect();

    // Qdrant counts
    let conscious = get_qdrant_count(&state.qdrant_url, "memories").await.unwrap_or(0);
    let unconscious = get_qdrant_count(&state.qdrant_url, "unconscious").await.unwrap_or(0);

    Ok(DashboardMetrics {
        timestamp: Utc::now(),
        identity: IdentityMetrics { name: "Timmy".into(), uptime_seconds: uptime, lifetime_thoughts, session_thoughts, restart_count },
        cognitive: CognitiveMetrics { conscious_memories: conscious, unconscious_memories: unconscious, lifetime_dreams, current_cycle: session_thoughts },
        emotional: EmotionalMetrics { valence: 0.0, arousal: 0.5, dominance: 0.5, connection_drive: 0.5, emotional_intensity: 0.0 },
        actors: ActorMetrics {
            memory_actor: ActorStatus { name: "MemoryActor".into(), alive: true, restart_count: 0 },
            attention_actor: ActorStatus { name: "AttentionActor".into(), alive: true, restart_count: 0 },
            salience_actor: ActorStatus { name: "SalienceActor".into(), alive: true, restart_count: 0 },
            volition_actor: ActorStatus { name: "VolitionActor".into(), alive: true, restart_count: 0 },
        },
        recent_thoughts,
    })
}

async fn get_qdrant_count(url: &str, collection: &str) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
    let client = qdrant_client::Qdrant::from_url(url).build()?;
    Ok(client.collection_info(collection).await?.result.map(|r| r.points_count.unwrap_or(0)).unwrap_or(0))
}

// =============================================================================
// Main
// =============================================================================

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().with_env_filter("daneel_web=info,tower_http=debug").init();
    dotenvy::dotenv().ok();

    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".into());
    let qdrant_url = std::env::var("QDRANT_URL").unwrap_or_else(|_| "http://localhost:6334".into());
    let port: u16 = std::env::var("PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(3000);

    info!("DANEEL Web Dashboard starting on port {}", port);
    let state = Arc::new(AppState::new(redis_url, qdrant_url));

    tokio::spawn(metrics_updater(Arc::clone(&state)));

    // Leptos WASM frontend from sibling project
    let frontend_dir = std::env::var("FRONTEND_DIR")
        .unwrap_or_else(|_| "../daneel-web-ui/dist".into());

    let app = Router::new()
        .route("/health", get(health))
        .route("/metrics", get(metrics))
        .route("/ws", get(ws_handler))
        .fallback_service(ServeDir::new(&frontend_dir))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    info!("Serving frontend from: {}", frontend_dir);

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port)).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_metrics() {
        let m = AppState::default_metrics();
        assert_eq!(m.identity.name, "Timmy");
    }

    #[test]
    fn test_serialization() {
        let m = AppState::default_metrics();
        assert!(serde_json::to_string(&m).is_ok());
    }
}
