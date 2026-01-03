# DANEEL-WEB - Observation Dashboard
# Multi-stage build: Rust backend + Leptos WASM frontend
#
# Build: docker build -t timmy-daneel-web .

# =============================================================================
# Stage 1: Build WASM frontend with Trunk
# =============================================================================
FROM rust:1.83-bookworm AS wasm-builder

# Install build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    binaryen \
    && rm -rf /var/lib/apt/lists/*

# Install trunk and wasm target
RUN cargo install trunk && \
    rustup target add wasm32-unknown-unknown

WORKDIR /app/frontend

# Copy frontend manifests
COPY frontend/Cargo.toml frontend/Cargo.lock* ./

# Create dummy lib for dependency caching
RUN mkdir src && echo "pub fn main() {}" > src/lib.rs

# Build frontend dependencies (ignore errors for dummy build)
RUN trunk build --release 2>/dev/null || true
RUN rm -rf src dist

# Copy actual frontend source
COPY frontend/src ./src
COPY frontend/index.html frontend/style.css frontend/Trunk.toml ./

# Build WASM bundle
RUN trunk build --release

# =============================================================================
# Stage 2: Build Rust backend
# =============================================================================
FROM rust:1.83-bookworm AS backend-builder

# Install build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy manifests first (dependency caching)
COPY Cargo.toml Cargo.lock ./

# Create dummy src for dependency build
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies (this layer is cached)
RUN cargo build --release && rm -rf src

# Copy actual source
COPY src ./src

# Build the real binary
RUN touch src/main.rs && cargo build --release

# Strip binary
RUN strip /app/target/release/daneel-web

# =============================================================================
# Stage 3: Runtime (minimal Debian)
# =============================================================================
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy backend binary
COPY --from=backend-builder /app/target/release/daneel-web /app/daneel-web

# Copy WASM frontend bundle
COPY --from=wasm-builder /app/frontend/dist /app/frontend/dist

# fastembed cache directory (mounted as volume)
RUN mkdir -p /root/.cache/fastembed

# Expose dashboard port
EXPOSE 3000

ENTRYPOINT ["/app/daneel-web"]
