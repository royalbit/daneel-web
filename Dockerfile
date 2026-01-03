# DANEEL-WEB - Observation Dashboard
# Runtime-only image (binary + frontend built externally)
#
# Build first:
#   make build                    # Backend
#   cd frontend && trunk build --release  # Frontend WASM
# Then: docker build -t timmy-daneel-web .

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy pre-built binary (built with `make build`)
COPY target/release/daneel-web /app/daneel-web

# Copy WASM frontend (built with `trunk build --release`)
COPY frontend/dist /app/frontend/dist

# fastembed cache directory (mounted as volume)
RUN mkdir -p /root/.cache/fastembed

EXPOSE 3000

ENTRYPOINT ["/app/daneel-web"]
