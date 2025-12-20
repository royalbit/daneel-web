# DANEEL Web Dashboard

**The Observable Mind** - A real-time nursery window into Timmy's cognitive processes.

## Overview

This is the web-based companion to DANEEL's TUI dashboard. While the TUI is the primary engineering interface, this dashboard provides a beautiful, "Hollywood-ready" view for observers who want to watch Timmy think without running a terminal.

## Features

- **Real-time updates** via WebSocket (200ms push interval)
- **Identity metrics**: Name, uptime, thought counts, restart count
- **Cognitive state**: Conscious/unconscious memory counts, dream cycles
- **Emotional state**: Valence, arousal, dominance (Russell's circumplex)
- **Connection Drive**: Real-time gauge showing kinship-weighted drive
- **Actor status**: Live view of cognitive actor health
- **Thought stream**: Last 20 thoughts with salience scores

## Security

**ALL ENDPOINTS ARE READ-ONLY.**

- No write access to Redis or Qdrant
- Asimov guardrails enforced at the proxy layer
- No public exposure by default (localhost only)

## Quick Start

```bash
# Set environment (optional, defaults shown)
export REDIS_URL=redis://localhost:6379
export QDRANT_URL=http://localhost:6334
export PORT=3000

# Run
cargo run
```

Then open http://localhost:3000

## API Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/` | GET | Dashboard HTML page |
| `/health` | GET | Health check (JSON) |
| `/metrics` | GET | Current metrics snapshot (JSON) |
| `/ws` | WS | Real-time metrics push |

## Roadmap

- **Phase 1** (current): Minimal viable dashboard with vanilla HTML/JS
- **Phase 2**: Hollywood layer with 3D brain visualization (three-rs)
- **Phase 3**: Vector fractals via Forge (PCA/t-SNE projections)

## License

AGPL-3.0-or-later - Same as DANEEL.

## Part of the DANEEL Family

- [daneel](https://github.com/royalbit/daneel) - The cognitive architecture
- [daneel-poster](https://github.com/royalbit/daneel-poster) - Social media automation
- daneel-web (this) - Web dashboard
