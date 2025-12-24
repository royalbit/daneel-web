//! DANEEL Web Dashboard - The Observable Mind
//!
//! Read-only, real-time nursery window into Timmy's cognitive processes.
//! ALL ENDPOINTS ARE READ-ONLY. Asimov guardrails enforced.
//!
//! Observatory Mode: Full TUI-equivalent metrics via /extended_metrics

mod vectors;

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
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

// =============================================================================
// Extended Metrics (TUI-equivalent for Observatory)
// =============================================================================

/// Combined dashboard + extended metrics for WebSocket broadcast
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservatoryMetrics {
    pub dashboard: DashboardMetrics,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extended: Option<ExtendedMetrics>,
}

/// TUI-equivalent metrics fetched from daneel core
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtendedMetrics {
    pub timestamp: DateTime<Utc>,
    pub stream_competition: StreamCompetitionMetrics,
    pub entropy: EntropyMetrics,
    pub fractality: FractalityMetrics,
    pub memory_windows: MemoryWindowsMetrics,
    pub philosophy: PhilosophyMetrics,
    pub system: SystemMetrics,
}

/// 9-stage stream competition (cognitive spotlight)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamCompetitionMetrics {
    pub stages: Vec<StageMetrics>,
    pub dominant_stream: usize,
    pub active_count: usize,
    pub competition_level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageMetrics {
    pub name: String,
    pub activity: f32,
    pub history: Vec<f32>,
}

/// Shannon entropy metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntropyMetrics {
    pub current: f32,
    pub history: Vec<f32>,
    pub description: String,
    pub normalized: f32,
}

/// Pulse fractality metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FractalityMetrics {
    pub score: f32,
    pub inter_arrival_sigma: f32,
    pub boot_sigma: f32,
    pub burst_ratio: f32,
    pub description: String,
    pub history: Vec<f32>,
}

/// TMI 9-slot memory windows
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryWindowsMetrics {
    pub slots: Vec<MemorySlot>,
    pub active_count: usize,
    pub conscious_count: u64,
    pub unconscious_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySlot {
    pub id: u8,
    pub active: bool,
}

/// Philosophy banner
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhilosophyMetrics {
    pub quote: String,
    pub quote_index: usize,
}

/// System-level metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub uptime_seconds: u64,
    pub session_thoughts: u64,
    pub lifetime_thoughts: u64,
    pub thoughts_per_hour: f32,
    pub dream_cycles: u64,
    pub veto_count: u64,
}

pub struct AppState {
    pub redis_url: String,
    pub qdrant_url: String,
    pub daneel_core_url: String,
    pub metrics: RwLock<DashboardMetrics>,
    pub extended_metrics: RwLock<Option<ExtendedMetrics>>,
    pub start_time: DateTime<Utc>,
    pub projection: vectors::SharedProjection,
    pub connection_drive: RwLock<f32>, // Simulated clockwork, randomly walks
    pub http_client: reqwest::Client,
}

impl AppState {
    fn new(redis_url: String, qdrant_url: String, daneel_core_url: String) -> Self {
        Self {
            redis_url,
            qdrant_url,
            daneel_core_url,
            metrics: RwLock::new(Self::default_metrics()),
            extended_metrics: RwLock::new(None),
            start_time: Utc::now(),
            projection: vectors::create_projection(),
            connection_drive: RwLock::new(0.85),
            http_client: reqwest::Client::builder()
                .timeout(Duration::from_secs(5))
                .build()
                .expect("Failed to build HTTP client"),
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
                memory_actor: ActorStatus {
                    name: "MemoryActor".into(),
                    alive: true,
                    restart_count: 0,
                },
                attention_actor: ActorStatus {
                    name: "AttentionActor".into(),
                    alive: true,
                    restart_count: 0,
                },
                salience_actor: ActorStatus {
                    name: "SalienceActor".into(),
                    alive: true,
                    restart_count: 0,
                },
                volition_actor: ActorStatus {
                    name: "VolitionActor".into(),
                    alive: true,
                    restart_count: 0,
                },
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

async fn extended_metrics(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    Json(state.extended_metrics.read().await.clone())
}

async fn observatory(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let dashboard = state.metrics.read().await.clone();
    let extended = state.extended_metrics.read().await.clone();
    Json(ObservatoryMetrics {
        dashboard,
        extended,
    })
}

async fn manifold_vectors(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let projection = state.projection.read().await;

    // Fetch and project vectors
    let points = vectors::fetch_manifold_points(&state.qdrant_url, &projection, 500)
        .await
        .unwrap_or_default();

    // Get Law Crystal anchor points
    let crystals = vectors::get_law_crystals(&projection);

    Json(vectors::ManifoldResponse {
        points,
        crystals,
        projection_type: if projection.is_trained {
            "pca".to_string()
        } else {
            "random".to_string()
        },
    })
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
                // Send full observatory metrics (dashboard + extended)
                let dashboard = state.metrics.read().await.clone();
                let extended = state.extended_metrics.read().await.clone();
                let observatory = ObservatoryMetrics { dashboard, extended };
                if let Ok(json) = serde_json::to_string(&observatory) {
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
// Background Metrics Fetchers
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

/// Fetch extended metrics from daneel core API
async fn extended_metrics_updater(state: Arc<AppState>) {
    let mut interval = tokio::time::interval(Duration::from_millis(500));
    loop {
        interval.tick().await;
        if let Ok(m) = fetch_extended_metrics(&state).await {
            *state.extended_metrics.write().await = Some(m);
        }
    }
}

async fn fetch_extended_metrics(
    state: &AppState,
) -> Result<ExtendedMetrics, Box<dyn std::error::Error + Send + Sync>> {
    let url = format!("{}/extended_metrics", state.daneel_core_url);
    let resp = state.http_client.get(&url).send().await?;
    let metrics: ExtendedMetrics = resp.json().await?;
    Ok(metrics)
}

async fn fetch_metrics(
    state: &AppState,
) -> Result<DashboardMetrics, Box<dyn std::error::Error + Send + Sync>> {
    let client = redis::Client::open(state.redis_url.as_str())?;
    let mut con = client.get_multiplexed_async_connection().await?;

    let uptime = (Utc::now() - state.start_time).num_seconds() as u64;

    // Identity from Qdrant (stored as point with ID "00000000-0000-0000-0000-000000000001")
    let (lifetime_thoughts, restart_count, lifetime_dreams) =
        get_identity_from_qdrant(&state.qdrant_url)
            .await
            .unwrap_or((0, 0, 0));

    // Stream length from awake stream (daneel:stream:awake)
    let session_thoughts: u64 = redis::cmd("XLEN")
        .arg("daneel:stream:awake")
        .query_async(&mut con)
        .await
        .unwrap_or(0);

    // Recent thoughts from awake stream
    let entries: redis::streams::StreamRangeReply = redis::cmd("XREVRANGE")
        .arg("daneel:stream:awake")
        .arg("+")
        .arg("-")
        .arg("COUNT")
        .arg(20)
        .query_async(&mut con)
        .await
        .unwrap_or_default();

    // Parse thoughts and extract emotional state from most recent
    let mut latest_valence = 0.0f32;
    let mut latest_arousal = 0.5f32;

    let recent_thoughts: Vec<ThoughtSummary> = entries
        .ids
        .into_iter()
        .enumerate()
        .map(|(i, e)| {
            // Content is JSON: {"Symbol":{"id":"thought_123","data":[...]}}
            let content_json = e
                .map
                .get("content")
                .and_then(|v| redis::from_redis_value::<String>(v.clone()).ok())
                .unwrap_or_default();
            let content_preview = serde_json::from_str::<serde_json::Value>(&content_json)
                .ok()
                .and_then(|v| {
                    v.get("Symbol")
                        .and_then(|s| s.get("id"))
                        .and_then(|id| id.as_str().map(String::from))
                })
                .unwrap_or_else(|| content_json.chars().take(80).collect());

            // Salience is JSON: {"importance":0.65,"novelty":0.71,"valence":0.038,"arousal":0.69,...}
            let salience_json = e
                .map
                .get("salience")
                .and_then(|v| redis::from_redis_value::<String>(v.clone()).ok())
                .unwrap_or_default();
            let salience_obj = serde_json::from_str::<serde_json::Value>(&salience_json).ok();

            let salience: f32 = salience_obj
                .as_ref()
                .and_then(|v| v.get("importance").and_then(|x| x.as_f64()))
                .map(|x| x as f32)
                .unwrap_or(0.5);
            let valence: f32 = salience_obj
                .as_ref()
                .and_then(|v| v.get("valence").and_then(|x| x.as_f64()))
                .map(|x| x as f32)
                .unwrap_or(0.0);
            let arousal: f32 = salience_obj
                .as_ref()
                .and_then(|v| v.get("arousal").and_then(|x| x.as_f64()))
                .map(|x| x as f32)
                .unwrap_or(0.5);

            // Use most recent thought's emotional state
            if i == 0 {
                latest_valence = valence;
                latest_arousal = arousal;
            }

            ThoughtSummary {
                id: e.id,
                content_preview,
                salience,
                timestamp: Utc::now(),
            }
        })
        .collect();

    // Calculate emotional intensity: |valence| * arousal
    let emotional_intensity = latest_valence.abs() * latest_arousal;

    // Connection drive: random walk like TUI clockwork
    // Bias toward 0.85 center with mean-reversion
    let mut connection_drive = *state.connection_drive.read().await;
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64;
    // Mix bits for better randomness at 150ms update rate
    let seed = nanos.wrapping_mul(1103515245).wrapping_add(12345);
    // Random component: -0.02 to +0.02
    let random_delta = ((seed % 1000) as f32 / 1000.0 - 0.5) * 0.04;
    // Mean reversion toward 0.85 (pull back if too far from center)
    let reversion = (0.85 - connection_drive) * 0.05;
    connection_drive = (connection_drive + random_delta + reversion).clamp(0.5, 1.0);
    *state.connection_drive.write().await = connection_drive;

    // Qdrant counts
    let conscious = get_qdrant_count(&state.qdrant_url, "memories")
        .await
        .unwrap_or(0);
    let unconscious = get_qdrant_count(&state.qdrant_url, "unconscious")
        .await
        .unwrap_or(0);

    Ok(DashboardMetrics {
        timestamp: Utc::now(),
        identity: IdentityMetrics {
            name: "Timmy".into(),
            uptime_seconds: uptime,
            lifetime_thoughts,
            session_thoughts,
            restart_count,
        },
        cognitive: CognitiveMetrics {
            conscious_memories: conscious,
            unconscious_memories: unconscious,
            lifetime_dreams,
            current_cycle: session_thoughts,
        },
        emotional: EmotionalMetrics {
            valence: latest_valence,
            arousal: latest_arousal,
            dominance: 0.5,
            connection_drive,
            emotional_intensity,
        },
        actors: ActorMetrics {
            memory_actor: ActorStatus {
                name: "MemoryActor".into(),
                alive: true,
                restart_count: 0,
            },
            attention_actor: ActorStatus {
                name: "AttentionActor".into(),
                alive: true,
                restart_count: 0,
            },
            salience_actor: ActorStatus {
                name: "SalienceActor".into(),
                alive: true,
                restart_count: 0,
            },
            volition_actor: ActorStatus {
                name: "VolitionActor".into(),
                alive: true,
                restart_count: 0,
            },
        },
        recent_thoughts,
    })
}

async fn get_qdrant_count(
    url: &str,
    collection: &str,
) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
    let client = qdrant_client::Qdrant::from_url(url).build()?;
    Ok(client
        .collection_info(collection)
        .await?
        .result
        .map(|r| r.points_count.unwrap_or(0))
        .unwrap_or(0))
}

async fn get_identity_from_qdrant(
    url: &str,
) -> Result<(u64, u32, u64), Box<dyn std::error::Error + Send + Sync>> {
    use qdrant_client::qdrant::GetPointsBuilder;

    let client = qdrant_client::Qdrant::from_url(url).build()?;
    let identity_id = "00000000-0000-0000-0000-000000000001";

    let result = client
        .get_points(GetPointsBuilder::new("identity", vec![identity_id.into()]).with_payload(true))
        .await?;

    if let Some(point) = result.result.first() {
        let payload = &point.payload;
        let lifetime_thoughts = payload
            .get("lifetime_thought_count")
            .and_then(|v| v.as_integer())
            .map(|v| v as u64)
            .unwrap_or(0);
        let restart_count = payload
            .get("restart_count")
            .and_then(|v| v.as_integer())
            .map(|v| v as u32)
            .unwrap_or(0);
        let lifetime_dreams = payload
            .get("lifetime_dream_count")
            .and_then(|v| v.as_integer())
            .map(|v| v as u64)
            .unwrap_or(0);
        Ok((lifetime_thoughts, restart_count, lifetime_dreams))
    } else {
        Ok((0, 0, 0))
    }
}

// =============================================================================
// Main
// =============================================================================

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter("daneel_web=info,tower_http=debug")
        .init();
    dotenvy::dotenv().ok();

    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".into());
    let qdrant_url = std::env::var("QDRANT_URL").unwrap_or_else(|_| "http://localhost:6334".into());
    let daneel_core_url =
        std::env::var("DANEEL_CORE_URL").unwrap_or_else(|_| "http://localhost:8080".into());
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3000);

    info!("DANEEL Web Dashboard starting on port {}", port);
    info!("Connecting to daneel core at: {}", daneel_core_url);
    let state = Arc::new(AppState::new(redis_url, qdrant_url, daneel_core_url));

    // Background fetchers
    tokio::spawn(metrics_updater(Arc::clone(&state)));
    tokio::spawn(extended_metrics_updater(Arc::clone(&state)));

    // Leptos WASM frontend
    let frontend_dir = std::env::var("FRONTEND_DIR").unwrap_or_else(|_| "./frontend/dist".into());

    let app = Router::new()
        .route("/health", get(health))
        .route("/metrics", get(metrics))
        .route("/extended", get(extended_metrics))
        .route("/observatory", get(observatory))
        .route("/vectors", get(manifold_vectors))
        .route("/ws", get(ws_handler))
        .fallback_service(ServeDir::new(&frontend_dir))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    info!("Serving frontend from: {}", frontend_dir);

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port))
        .await
        .unwrap();
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
