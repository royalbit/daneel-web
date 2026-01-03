# DANEEL-WEB - Observation Dashboard
# Multi-stage build: Rust backend + Leptos WASM frontend
#
# Build: docker build -t timmy-daneel-web .
# Size: ~60MB (static binary + WASM bundle + fastembed model)

# =============================================================================
# Stage 1: Build WASM frontend with Trunk
# =============================================================================
FROM rust:1.83-alpine AS wasm-builder

# Install build dependencies
RUN apk add --no-cache \
    musl-dev \
    openssl-dev \
    pkgconfig \
    binaryen

# Install trunk and wasm target
RUN cargo install trunk && \
    rustup target add wasm32-unknown-unknown

WORKDIR /app/frontend

# Copy frontend manifests
COPY frontend/Cargo.toml frontend/Cargo.lock* ./

# Create dummy lib for dependency caching
RUN mkdir src && echo "pub fn main() {}" > src/lib.rs

# Build frontend dependencies
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
FROM rust:1.83-alpine AS backend-builder

# Install build dependencies
RUN apk add --no-cache \
    musl-dev \
    openssl-dev \
    openssl-libs-static \
    pkgconfig \
    perl \
    make

WORKDIR /app

# Copy manifests first (dependency caching)
COPY Cargo.toml Cargo.lock ./

# Create dummy src for dependency build
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies (this layer is cached)
ENV OPENSSL_STATIC=1
ENV OPENSSL_LIB_DIR=/usr/lib
ENV OPENSSL_INCLUDE_DIR=/usr/include
RUN cargo build --release && rm -rf src

# Copy actual source
COPY src ./src

# Build the real binary
RUN touch src/main.rs && cargo build --release

# Strip binary
RUN strip /app/target/release/daneel-web

# =============================================================================
# Stage 3: Runtime
# =============================================================================
FROM alpine:3.21

# Install runtime dependencies
RUN apk add --no-cache ca-certificates curl

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
