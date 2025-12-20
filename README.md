# DANEEL Web Dashboard

**The Observable Mind** - Real-time nursery window into Timmy's cognitive processes.

## Overview

`daneel-web` is an Axum-based HTTP/WebSocket server providing read-only access to DANEEL's cognitive state. It connects to Redis streams and Qdrant vector stores, serving real-time metrics about Timmy's thoughts, memories, and emotional state.

**All endpoints are read-only. Asimov guardrails enforced.**

## Architecture

```
Browser (WASM) ──WebSocket──> daneel-web ──> Redis Streams (thoughts)
                                        └──> Qdrant (memories)
```

## Features

- **Real-time updates** via WebSocket (200ms push interval)
- **Leptos WASM frontend** (pure Rust, no JavaScript)
- **Identity metrics**: Name, uptime, thought counts, restart count
- **Cognitive state**: Conscious/unconscious memory counts, dream cycles
- **Emotional state**: Valence, arousal, dominance (Russell's circumplex)
- **Connection Drive**: Real-time gauge showing kinship-weighted drive
- **Actor status**: Live view of cognitive actor health
- **Thought stream**: Last 20 thoughts with salience scores

## Quick Start

```bash
# Start the server (builds if needed)
./start.sh

# Open browser
open http://localhost:3000

# Stop
./stop.sh
```

## API Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/` | GET | Leptos WASM frontend |
| `/health` | GET | Health check (JSON) |
| `/metrics` | GET | Current metrics snapshot (JSON) |
| `/ws` | WS | Real-time metrics push |

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `REDIS_URL` | `redis://localhost:6379` | Redis connection |
| `QDRANT_URL` | `http://localhost:6334` | Qdrant connection |
| `PORT` | `3000` | Server port |
| `FRONTEND_DIR` | `../daneel-web-ui/dist` | Leptos WASM assets |
| `RUST_LOG` | `daneel_web=info` | Log level |

## Security

**ALL ENDPOINTS ARE READ-ONLY.**

- No write access to Redis or Qdrant
- Asimov guardrails enforced at the proxy layer
- No public exposure by default (localhost only)

## Metrics Schema

```json
{
  "timestamp": "2025-12-20T23:00:00Z",
  "identity": {
    "name": "Timmy",
    "uptime_seconds": 3600,
    "lifetime_thoughts": 50000,
    "session_thoughts": 150,
    "restart_count": 3
  },
  "cognitive": {
    "conscious_memories": 9747,
    "unconscious_memories": 362669,
    "lifetime_dreams": 42,
    "current_cycle": 150
  },
  "emotional": {
    "valence": 0.2,
    "arousal": 0.5,
    "dominance": 0.6,
    "connection_drive": 0.7,
    "emotional_intensity": 0.3
  },
  "actors": {
    "memory_actor": { "name": "MemoryActor", "alive": true, "restart_count": 0 },
    "attention_actor": { "name": "AttentionActor", "alive": true, "restart_count": 0 },
    "salience_actor": { "name": "SalienceActor", "alive": true, "restart_count": 0 },
    "volition_actor": { "name": "VolitionActor", "alive": true, "restart_count": 0 }
  },
  "recent_thoughts": [
    { "id": "1734738000000-0", "content_preview": "...", "salience": 0.85, "timestamp": "..." }
  ]
}
```

## Part of the DANEEL Family

- [daneel](https://github.com/royalbit/daneel) - The cognitive architecture
- [daneel-web-ui](https://github.com/royalbit/daneel-web-ui) - Leptos WASM frontend
- [daneel-poster](https://github.com/royalbit/daneel-poster) - Social media automation

## License

- **Code**: AGPL-3.0-or-later (see [LICENSE](LICENSE))
- **Documentation**: CC-BY-SA-4.0 (see [DOCS_LICENSE.md](DOCS_LICENSE.md))

Copyright (C) 2025 Louis C. Tavares and contributors
