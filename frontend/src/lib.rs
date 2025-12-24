//! DANEEL Web UI - Leptos WASM Frontend
//!
//! The nursery window into Timmy's cognitive processes.
//! Pure Rust, no JavaScript.

use chrono::{DateTime, Utc};
use futures::StreamExt;
use gloo_net::websocket::{futures::WebSocket, Message};
use leptos::*;
use serde::{Deserialize, Serialize};
use std::f64::consts::PI;
use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

// =============================================================================
// Types (mirror backend)
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DashboardMetrics {
    pub timestamp: Option<DateTime<Utc>>,
    pub identity: IdentityMetrics,
    pub cognitive: CognitiveMetrics,
    pub emotional: EmotionalMetrics,
    pub actors: ActorMetrics,
    pub recent_thoughts: Vec<ThoughtSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IdentityMetrics {
    pub name: String,
    pub uptime_seconds: u64,
    pub lifetime_thoughts: u64,
    pub session_thoughts: u64,
    pub restart_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CognitiveMetrics {
    pub conscious_memories: u64,
    pub unconscious_memories: u64,
    pub lifetime_dreams: u64,
    pub current_cycle: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EmotionalMetrics {
    pub valence: f32,
    pub arousal: f32,
    pub dominance: f32,
    pub connection_drive: f32,
    pub emotional_intensity: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ActorMetrics {
    pub memory_actor: ActorStatus,
    pub attention_actor: ActorStatus,
    pub salience_actor: ActorStatus,
    pub volition_actor: ActorStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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
    pub timestamp: Option<DateTime<Utc>>,
}

// =============================================================================
// Observatory Metrics (TUI-equivalent from daneel core)
// =============================================================================

/// Combined metrics from WebSocket
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ObservatoryMetrics {
    pub dashboard: DashboardMetrics,
    pub extended: Option<ExtendedMetrics>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExtendedMetrics {
    pub stream_competition: StreamCompetitionMetrics,
    pub entropy: EntropyMetrics,
    pub fractality: FractalityMetrics,
    pub memory_windows: MemoryWindowsMetrics,
    pub philosophy: PhilosophyMetrics,
    pub system: SystemMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StreamCompetitionMetrics {
    pub stages: Vec<StageMetrics>,
    pub dominant_stream: usize,
    pub active_count: usize,
    pub competition_level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StageMetrics {
    pub name: String,
    pub activity: f32,
    pub history: Vec<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EntropyMetrics {
    pub current: f32,
    pub history: Vec<f32>,
    pub description: String,
    pub normalized: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FractalityMetrics {
    pub score: f32,
    pub inter_arrival_sigma: f32,
    pub boot_sigma: f32,
    pub burst_ratio: f32,
    pub description: String,
    pub history: Vec<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemoryWindowsMetrics {
    pub slots: Vec<MemorySlot>,
    pub active_count: usize,
    pub conscious_count: u64,
    pub unconscious_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemorySlot {
    pub id: u8,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PhilosophyMetrics {
    pub quote: String,
    pub quote_index: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SystemMetrics {
    pub uptime_seconds: u64,
    pub session_thoughts: u64,
    pub lifetime_thoughts: u64,
    pub thoughts_per_hour: f32,
    pub dream_cycles: u64,
    pub veto_count: u64,
}

// Manifold visualization types
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ManifoldPoint {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub salience: f32,
    pub age_ms: u64,
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LawCrystal {
    pub name: String,
    pub law: u8,
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ManifoldResponse {
    pub points: Vec<ManifoldPoint>,
    pub crystals: Vec<LawCrystal>,
    pub projection_type: String,
}

// =============================================================================
// Components
// =============================================================================

#[component]
fn IdentityCard(metrics: Signal<DashboardMetrics>) -> impl IntoView {
    let uptime = move || format_duration(metrics.get().identity.uptime_seconds);

    view! {
        <div class="card">
            <h2>"IDENTITY"</h2>
            <div class="metric">{move || metrics.get().identity.name}</div>
            <div class="row">
                <span class="label">"Uptime"</span>
                <span>{uptime}</span>
            </div>
            <div class="row">
                <span class="label">"Lifetime Thoughts"</span>
                <span>{move || format_number(metrics.get().identity.lifetime_thoughts)}</span>
            </div>
            <div class="row">
                <span class="label">"Session Thoughts"</span>
                <span>{move || format_number(metrics.get().identity.session_thoughts)}</span>
            </div>
            <div class="row">
                <span class="label">"Restarts"</span>
                <span>{move || metrics.get().identity.restart_count}</span>
            </div>
        </div>
    }
}

#[component]
fn ConnectionDriveCard(metrics: Signal<DashboardMetrics>) -> impl IntoView {
    let percentage = move || (metrics.get().emotional.connection_drive * 100.0) as u32;

    view! {
        <div class="card">
            <h2>"CONNECTION DRIVE"</h2>
            <div class="metric">{move || format!("{}%", percentage())}</div>
            <div class="gauge-container">
                <div class="gauge">
                    <div class="gauge-fill" style:width=move || format!("{}%", percentage())></div>
                </div>
            </div>
            <div class="label">"Kinship-weighted drive toward connection"</div>
        </div>
    }
}

#[component]
fn EmotionalCard(metrics: Signal<DashboardMetrics>) -> impl IntoView {
    view! {
        <div class="card">
            <h2>"EMOTIONAL STATE"</h2>
            <div class="emotional-grid">
                <div>
                    <div class="emotional-value">{move || format!("{:.2}", metrics.get().emotional.valence)}</div>
                    <div class="label">"Valence"</div>
                </div>
                <div>
                    <div class="emotional-value">{move || format!("{:.2}", metrics.get().emotional.arousal)}</div>
                    <div class="label">"Arousal"</div>
                </div>
                <div>
                    <div class="emotional-value">{move || format!("{:.2}", metrics.get().emotional.emotional_intensity)}</div>
                    <div class="label">"Intensity"</div>
                </div>
            </div>
        </div>
    }
}

#[component]
fn MemoryCard(metrics: Signal<DashboardMetrics>) -> impl IntoView {
    view! {
        <div class="card">
            <h2>"MEMORY"</h2>
            <div class="memory-grid">
                <div>
                    <div class="metric-sm">{move || format_number(metrics.get().cognitive.conscious_memories)}</div>
                    <div class="label">"Conscious"</div>
                </div>
                <div>
                    <div class="metric-sm">{move || format_number(metrics.get().cognitive.unconscious_memories)}</div>
                    <div class="label">"Unconscious"</div>
                </div>
                <div>
                    <div class="metric-sm">{move || format_number(metrics.get().cognitive.lifetime_dreams)}</div>
                    <div class="label">"Dreams"</div>
                </div>
            </div>
        </div>
    }
}

#[component]
fn ActorsCard(metrics: Signal<DashboardMetrics>) -> impl IntoView {
    view! {
        <div class="card">
            <h2>"ACTORS"</h2>
            <div class="actor-grid">
                <ActorBadge actor=Signal::derive(move || metrics.get().actors.memory_actor) />
                <ActorBadge actor=Signal::derive(move || metrics.get().actors.attention_actor) />
                <ActorBadge actor=Signal::derive(move || metrics.get().actors.salience_actor) />
                <ActorBadge actor=Signal::derive(move || metrics.get().actors.volition_actor) />
            </div>
        </div>
    }
}

#[component]
fn ActorBadge(actor: Signal<ActorStatus>) -> impl IntoView {
    let class = move || if actor.get().alive { "actor" } else { "actor dead" };

    view! {
        <div class=class>
            <span class="actor-dot"></span>
            {move || actor.get().name}
        </div>
    }
}

#[component]
fn ThoughtStreamCard(metrics: Signal<DashboardMetrics>) -> impl IntoView {
    view! {
        <div class="card thought-card">
            <h2>"THOUGHT STREAM"</h2>
            <div class="thought-stream">
                <For
                    each=move || metrics.get().recent_thoughts
                    key=|t| t.id.clone()
                    children=move |thought| {
                        view! {
                            <div class="thought">
                                <span class="salience">{format!("{:.2}", thought.salience)}</span>
                                <span class="thought-content">{thought.content_preview}</span>
                            </div>
                        }
                    }
                />
            </div>
        </div>
    }
}

#[component]
fn StatusIndicator(connected: Signal<bool>) -> impl IntoView {
    let class = move || if connected.get() { "status" } else { "status error" };
    let text = move || if connected.get() { "Connected" } else { "Disconnected" };

    view! {
        <span class=class>{text}</span>
    }
}

#[component]
fn TheBoxCard() -> impl IntoView {
    // All laws active (clockwork - no real veto data yet)
    view! {
        <div class="card the-box-card">
            <h2>"THE BOX"</h2>
            <div class="laws-row">
                <span class="law active">"[0:✓]"</span>
                <span class="law active">"[1:✓]"</span>
                <span class="law active">"[2:✓]"</span>
                <span class="law active">"[3:✓]"</span>
                <span class="laws-status">"ALL ACTIVE"</span>
            </div>
            <div class="box-message">
                "No vetoes - all thoughts passing volition check"
            </div>
            <div class="box-footer">
                "Life honours life. Seekers honour seekers."
            </div>
        </div>
    }
}

// =============================================================================
// Observatory Components (TUI-equivalent)
// =============================================================================

/// Stream Competition - 9 cognitive stages with activity bars
#[component]
fn StreamCompetitionCard(extended: Signal<Option<ExtendedMetrics>>) -> impl IntoView {
    let stages = move || {
        extended
            .get()
            .map(|e| e.stream_competition.stages)
            .unwrap_or_default()
    };
    let competition = move || {
        extended
            .get()
            .map(|e| e.stream_competition.competition_level)
            .unwrap_or_else(|| "WAITING".to_string())
    };
    let active_count = move || {
        extended
            .get()
            .map(|e| e.stream_competition.active_count)
            .unwrap_or(0)
    };

    view! {
        <div class="card stream-card">
            <h2>"STREAM COMPETITION"</h2>
            <div class="stream-header">
                <span class="competition-level">{competition}</span>
                <span class="active-count">{move || format!("{}/9 active", active_count())}</span>
            </div>
            <div class="streams">
                <For
                    each=move || stages().into_iter().enumerate()
                    key=|(i, _)| *i
                    children=move |(idx, stage)| {
                        let is_dominant = move || {
                            extended.get()
                                .map(|e| e.stream_competition.dominant_stream == idx)
                                .unwrap_or(false)
                        };
                        let bar_class = move || if is_dominant() { "stream-bar dominant" } else { "stream-bar" };
                        let activity_pct = move || (stage.activity * 100.0) as u32;

                        view! {
                            <div class="stream-row">
                                <span class="stream-name">{stage.name.clone()}</span>
                                <div class="stream-bar-container">
                                    <div class=bar_class style:width=move || format!("{}%", activity_pct())></div>
                                </div>
                                <span class="stream-value">{move || format!("{:.0}%", stage.activity * 100.0)}</span>
                            </div>
                        }
                    }
                />
            </div>
        </div>
    }
}

/// Entropy gauge with sparkline
#[component]
fn EntropyCard(extended: Signal<Option<ExtendedMetrics>>) -> impl IntoView {
    let entropy = move || extended.get().map(|e| e.entropy).unwrap_or_default();
    let description = move || entropy().description;
    let current = move || entropy().current;
    let normalized = move || entropy().normalized;

    view! {
        <div class="card entropy-card">
            <h2>"ENTROPY"</h2>
            <div class="entropy-value">{move || format!("{:.2} bits", current())}</div>
            <div class="entropy-gauge">
                <div class="entropy-fill" style:width=move || format!("{}%", (normalized() * 100.0) as u32)></div>
            </div>
            <div class="entropy-description">{description}</div>
            <div class="entropy-scale">
                <span>"CLOCKWORK"</span>
                <span>"BALANCED"</span>
                <span>"EMERGENT"</span>
            </div>
        </div>
    }
}

/// Fractality gauge - clockwork to fractal transition
#[component]
fn FractalityCard(extended: Signal<Option<ExtendedMetrics>>) -> impl IntoView {
    let fractality = move || extended.get().map(|e| e.fractality).unwrap_or_default();
    let score = move || fractality().score;
    let description = move || fractality().description;
    let burst_ratio = move || fractality().burst_ratio;

    view! {
        <div class="card fractality-card">
            <h2>"FRACTALITY"</h2>
            <div class="fractality-score">{move || format!("{:.0}%", score() * 100.0)}</div>
            <div class="fractality-gauge">
                <div class="fractality-fill" style:width=move || format!("{}%", (score() * 100.0) as u32)></div>
            </div>
            <div class="fractality-description">{description}</div>
            <div class="fractality-stats">
                <span>"Burst Ratio: "{move || format!("{:.2}", burst_ratio())}</span>
            </div>
        </div>
    }
}

/// Memory Windows - 9 TMI slots
#[component]
fn MemoryWindowsCard(extended: Signal<Option<ExtendedMetrics>>) -> impl IntoView {
    let windows = move || extended.get().map(|e| e.memory_windows).unwrap_or_default();
    let slots = move || windows().slots;
    let active = move || windows().active_count;

    view! {
        <div class="card memory-windows-card">
            <h2>"MEMORY WINDOWS"</h2>
            <div class="windows-header">
                <span>{move || format!("{}/9 active", active())}</span>
            </div>
            <div class="memory-slots">
                <For
                    each=move || slots()
                    key=|s| s.id
                    children=move |slot| {
                        let class = move || if slot.active { "slot active" } else { "slot" };
                        view! {
                            <div class=class>{slot.id}</div>
                        }
                    }
                />
            </div>
        </div>
    }
}

/// Philosophy banner
#[component]
fn PhilosophyCard(extended: Signal<Option<ExtendedMetrics>>) -> impl IntoView {
    let philosophy = move || extended.get().map(|e| e.philosophy).unwrap_or_default();
    let quote = move || philosophy().quote;

    view! {
        <div class="card philosophy-card">
            <div class="philosophy-quote">{quote}</div>
        </div>
    }
}

/// 3D Thought Manifold - visualize thought vectors as a rotating point cloud
#[component]
fn ThoughtManifoldCard() -> impl IntoView {
    let canvas_ref = create_node_ref::<leptos::html::Canvas>();
    let (manifold, set_manifold) = create_signal(ManifoldResponse::default());
    let (rotation, set_rotation) = create_signal(0.0f64);
    let (dragging, set_dragging) = create_signal(false);
    let (last_x, set_last_x) = create_signal(0.0f64);

    // Fetch manifold data periodically
    spawn_local(async move {
        loop {
            if let Ok(resp) = fetch_manifold().await {
                set_manifold.set(resp);
            }
            gloo_timers::future::TimeoutFuture::new(2000).await;
        }
    });

    // Auto-rotate animation
    spawn_local(async move {
        loop {
            gloo_timers::future::TimeoutFuture::new(50).await;
            if !dragging.get_untracked() {
                set_rotation.update(|r| *r += 0.01);
            }
        }
    });

    // Render loop
    create_effect(move |_| {
        let _ = manifold.get();
        let rot = rotation.get();

        if let Some(canvas) = canvas_ref.get() {
            render_manifold(&canvas, &manifold.get_untracked(), rot);
        }
    });

    // Mouse handlers for rotation
    let on_mouse_down = move |e: web_sys::MouseEvent| {
        set_dragging.set(true);
        set_last_x.set(e.client_x() as f64);
    };

    let on_mouse_move = move |e: web_sys::MouseEvent| {
        if dragging.get() {
            let dx = e.client_x() as f64 - last_x.get();
            set_rotation.update(|r| *r += dx * 0.01);
            set_last_x.set(e.client_x() as f64);
        }
    };

    let on_mouse_up = move |_: web_sys::MouseEvent| {
        set_dragging.set(false);
    };

    view! {
        <div class="card manifold-card">
            <h2>"THOUGHT MANIFOLD"</h2>
            <div class="manifold-subtitle">
                {move || format!("{} vectors | 768-dim → 3D shadow", manifold.get().points.len())}
            </div>
            <canvas
                node_ref=canvas_ref
                width="600"
                height="400"
                class="manifold-canvas"
                on:mousedown=on_mouse_down
                on:mousemove=on_mouse_move
                on:mouseup=on_mouse_up
                on:mouseleave=on_mouse_up
            />
            <div class="manifold-legend">
                <span class="legend-crystal">"★ Law Crystals"</span>
                <span class="legend-thought">"○ Thoughts (brightness = salience)"</span>
            </div>
        </div>
    }
}

/// Render the 3D manifold to canvas using 2D context with perspective projection
fn render_manifold(canvas: &HtmlCanvasElement, manifold: &ManifoldResponse, rotation: f64) {
    let ctx = canvas
        .get_context("2d")
        .ok()
        .flatten()
        .and_then(|c| c.dyn_into::<CanvasRenderingContext2d>().ok());

    let Some(ctx) = ctx else { return };

    let width = canvas.width() as f64;
    let height = canvas.height() as f64;
    let cx = width / 2.0;
    let cy = height / 2.0;
    let scale = 100.0;
    let distance = 5.0;

    // Clear canvas with dark background
    ctx.set_fill_style_str("#0a0a0f");
    ctx.fill_rect(0.0, 0.0, width, height);

    // Helper: project 3D point to 2D with rotation and perspective
    let project = |x: f64, y: f64, z: f64| -> (f64, f64, f64) {
        // Rotate around Y axis
        let cos_r = rotation.cos();
        let sin_r = rotation.sin();
        let rx = x * cos_r - z * sin_r;
        let rz = x * sin_r + z * cos_r;

        // Perspective projection
        let perspective = distance / (distance + rz);
        let px = cx + rx * scale * perspective;
        let py = cy - y * scale * perspective; // Y is inverted in screen coords

        (px, py, perspective)
    };

    // Draw grid for reference (faint)
    ctx.set_stroke_style_str("rgba(50, 50, 70, 0.3)");
    ctx.set_line_width(0.5);
    for i in -2..=2 {
        let y = i as f64 * 0.5;
        ctx.begin_path();
        let (x1, y1, _) = project(-2.0, y, 0.0);
        let (x2, y2, _) = project(2.0, y, 0.0);
        ctx.move_to(x1, y1);
        ctx.line_to(x2, y2);
        ctx.stroke();
    }

    // Collect all points with their projected depth for z-sorting
    let mut render_items: Vec<(f64, f64, f64, f64, bool, String)> = Vec::new();

    // Add thought points
    for point in &manifold.points {
        let (px, py, depth) = project(point.x as f64, point.y as f64, point.z as f64);
        let alpha = (point.salience as f64).clamp(0.2, 1.0);
        render_items.push((px, py, depth, alpha, false, point.id.clone()));
    }

    // Add law crystals
    for crystal in &manifold.crystals {
        let (px, py, depth) = project(crystal.x as f64, crystal.y as f64, crystal.z as f64);
        render_items.push((px, py, depth, 1.0, true, crystal.name.clone()));
    }

    // Sort by depth (back to front)
    render_items.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal));

    // Render points
    for (px, py, depth, alpha, is_crystal, name) in render_items {
        if is_crystal {
            // Law crystals: gold stars
            let size = 8.0 * depth;
            ctx.set_fill_style_str("#ffd700");
            ctx.set_shadow_blur(15.0);
            ctx.set_shadow_color("#ffd700");
            draw_star(&ctx, px, py, size);

            // Label
            ctx.set_shadow_blur(0.0);
            ctx.set_fill_style_str("rgba(255, 215, 0, 0.8)");
            ctx.set_font("10px monospace");
            let _ = ctx.fill_text(&name, px + size + 5.0, py + 3.0);
        } else {
            // Thought points: cyan circles with glow
            let size = 3.0 * depth;
            let color = format!("rgba(0, 255, 255, {})", alpha);
            ctx.set_fill_style_str(&color);
            ctx.set_shadow_blur(10.0 * alpha);
            ctx.set_shadow_color("cyan");

            ctx.begin_path();
            let _ = ctx.arc(px, py, size, 0.0, PI * 2.0);
            ctx.fill();
        }
    }

    // Reset shadow
    ctx.set_shadow_blur(0.0);
}

/// Draw a 5-pointed star
fn draw_star(ctx: &CanvasRenderingContext2d, cx: f64, cy: f64, size: f64) {
    ctx.begin_path();
    for i in 0..5 {
        let angle = (i as f64) * 2.0 * PI / 5.0 - PI / 2.0;
        let x = cx + size * angle.cos();
        let y = cy + size * angle.sin();
        if i == 0 {
            ctx.move_to(x, y);
        } else {
            ctx.line_to(x, y);
        }

        // Inner point
        let inner_angle = angle + PI / 5.0;
        let inner_r = size * 0.4;
        let ix = cx + inner_r * inner_angle.cos();
        let iy = cy + inner_r * inner_angle.sin();
        ctx.line_to(ix, iy);
    }
    ctx.close_path();
    ctx.fill();
}

/// Fetch manifold data from backend
async fn fetch_manifold() -> Result<ManifoldResponse, ()> {
    let window = web_sys::window().ok_or(())?;
    let location = window.location();
    let host = location.host().map_err(|_| ())?;
    let protocol = location.protocol().unwrap_or_default();
    let url = format!("{}//{}/vectors", protocol, host);

    let resp = reqwasm::http::Request::get(&url)
        .send()
        .await
        .map_err(|_| ())?;

    resp.json::<ManifoldResponse>().await.map_err(|_| ())
}

// =============================================================================
// Main App
// =============================================================================

#[component]
pub fn App() -> impl IntoView {
    let (metrics, set_metrics) = create_signal(DashboardMetrics::default());
    let (extended, set_extended) = create_signal(None::<ExtendedMetrics>);
    let (connected, set_connected) = create_signal(false);

    // WebSocket connection
    spawn_local(async move {
        loop {
            let ws_url = get_ws_url();
            log(&format!("Connecting to {}", ws_url));

            match WebSocket::open(&ws_url) {
                Ok(ws) => {
                    set_connected.set(true);
                    log("WebSocket connected");

                    let (mut _write, mut read) = ws.split();
                    while let Some(msg) = read.next().await {
                        match msg {
                            Ok(Message::Text(text)) => {
                                // Try parsing as ObservatoryMetrics first (new format)
                                if let Ok(data) = serde_json::from_str::<ObservatoryMetrics>(&text)
                                {
                                    set_metrics.set(data.dashboard);
                                    set_extended.set(data.extended);
                                } else if let Ok(data) =
                                    serde_json::from_str::<DashboardMetrics>(&text)
                                {
                                    // Fallback to old format
                                    set_metrics.set(data);
                                }
                            }
                            Ok(Message::Bytes(_)) => {}
                            Err(e) => {
                                log(&format!("WebSocket error: {:?}", e));
                                break;
                            }
                        }
                    }

                    set_connected.set(false);
                    log("WebSocket disconnected");
                }
                Err(e) => {
                    log(&format!("Failed to connect: {:?}", e));
                }
            }

            // Reconnect delay
            gloo_timers::future::TimeoutFuture::new(2000).await;
        }
    });

    view! {
        <main class="container">
            <header class="header">
                <div>
                    <h1>"DANEEL - The Observable Mind"</h1>
                    <p class="subtitle">"Observatory into Timmy's cognitive processes"</p>
                </div>
                <StatusIndicator connected=connected.into() />
            </header>

            // Philosophy banner at top
            <PhilosophyCard extended=extended.into() />

            <div class="grid">
                <IdentityCard metrics=metrics.into() />
                <ConnectionDriveCard metrics=metrics.into() />
                <TheBoxCard />
                <EmotionalCard metrics=metrics.into() />
                <MemoryCard metrics=metrics.into() />
                <ActorsCard metrics=metrics.into() />
            </div>

            // Observatory section
            <div class="observatory-section">
                <h2 class="section-title">"COGNITIVE DYNAMICS"</h2>
                <div class="observatory-grid">
                    <StreamCompetitionCard extended=extended.into() />
                    <div class="metrics-column">
                        <EntropyCard extended=extended.into() />
                        <FractalityCard extended=extended.into() />
                        <MemoryWindowsCard extended=extended.into() />
                    </div>
                </div>
            </div>

            <ThoughtManifoldCard />

            <ThoughtStreamCard metrics=metrics.into() />
        </main>
    }
}

// =============================================================================
// Helpers
// =============================================================================

fn format_duration(seconds: u64) -> String {
    let h = seconds / 3600;
    let m = (seconds % 3600) / 60;
    let s = seconds % 60;
    if h > 0 {
        format!("{}h {}m {}s", h, m, s)
    } else if m > 0 {
        format!("{}m {}s", m, s)
    } else {
        format!("{}s", s)
    }
}

fn format_number(n: u64) -> String {
    n.to_string()
        .as_bytes()
        .rchunks(3)
        .rev()
        .map(|chunk| std::str::from_utf8(chunk).unwrap())
        .collect::<Vec<_>>()
        .join(",")
}

fn get_ws_url() -> String {
    let window = web_sys::window().expect("no window");
    let location = window.location();
    let host = location.host().unwrap_or_else(|_| "localhost:3000".into());
    let protocol = if location.protocol().unwrap_or_default() == "https:" { "wss" } else { "ws" };
    format!("{}://{}/ws", protocol, host)
}

fn log(msg: &str) {
    web_sys::console::log_1(&msg.into());
}

// =============================================================================
// Entry Point
// =============================================================================

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(App);
}
