# 🧡 胶带 (Jiaodai)

> 现在，封存。条件到了，才开。——时间封存平台

**Seal now, open when conditions met.**

## What is Jiaodai?

Jiaodai (胶带, meaning "tape") is a time-sealing platform that lets you seal content now and only unseal it when specific conditions are met. Like sealing an envelope with wax — the seal proves nobody tampered with it, and it only opens when the time is right.

### Core Three Elements

Every sealed tape consists of three elements:

1. **Sealable** (封存物) — The content: text, images, videos, files
2. **TriggerCondition** (解封条件) — When to open: heartbeat timeout, mutual match, date trigger, multi-person confirmation
3. **Viewer** (查看人) — Who can see: specific accounts, phone number holders, identity-verified persons, or anyone

### Scenarios

| Scenario | Content | Unseal Condition | Viewer |
|----------|---------|-----------------|--------|
| Last Will | Will/Instructions/Asset list | Heartbeat lost N days | Legal heirs |
| Secret Crush | Love letter | Mutual match | The other person |
| Time Capsule | Any content | Specific date | Self/designated |
| Graduation Message | Blessings/Photos | Multi-person confirmation | Classmates |

## Architecture

```
jiaodai/
├── crates/
│   ├── jiaodai-core/     # Core traits + data models
│   ├── jiaodai-seal/     # Sealing core (encryption + hash + certificate)
│   ├── jiaodai-unseal/   # Unseal engine (condition state machine + triggers)
│   ├── jiaodai-match/    # Bidirectional seal matching engine
│   ├── jiaodai-chain/    # Blockchain timestamp (L2 interaction)
│   ├── jiaodai-auth/     # Account system (register/login/JWT/identity verification)
│   ├── jiaodai-scene/    # Scenario implementations (crush/will/capsule)
│   ├── jiaodai-api/      # Axum HTTP API + WebSocket + CORS
│   └── jiaodai-cli/      # CLI entry point
```

## Quick Start

### Prerequisites

- Rust 1.75+ (edition 2021)
- SQLite 3.x

### Build & Run

```bash
# Build
cargo build

# Run tests
cargo test

# Start the server
cargo run -p jiaodai-cli
```

The server starts on `http://0.0.0.0:3000`.

### API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/v1/health` | Health check |
| POST | `/api/v1/seal` | Create a seal |
| POST | `/api/v1/unseal/:id` | Trigger unseal |
| GET | `/api/v1/tape/:id/status` | Check tape status |
| GET | `/api/v1/tape/:id/verify` | Verify tape hash |
| POST | `/api/v1/account/register` | Register account |
| POST | `/api/v1/account/login` | Login |
| POST | `/api/v1/heartbeat/confirm` | Confirmer heartbeat |
| GET | `/api/v1/match/check` | Check mutual match |
| GET | `/.well-known/agent.json` | Agent discovery |

### Docker

```bash
docker compose up -d
```

## Tech Stack

| Layer | Choice | Reason |
|-------|--------|--------|
| Language | Rust | Performance + safety |
| Web Framework | Axum | Async + type-safe |
| Database | SQLite / PostgreSQL | Lightweight start, scale when needed |
| Blockchain | Ethereum L2 | Low gas + secure |
| Encryption | AES-256-GCM + SHA-256 | Auditable, pure Rust |
| Key Splitting | Shamir's Secret Sharing | M-of-N key recovery |

## Project Status

**Phase 1-12 Complete** ✅ — v0.1.0, 9 crate Rust workspace, 177 tests all green, cross-platform CI + Release.

| Phase | Status | Description |
|-------|--------|-------------|
| Phase 1 | ✅ | Project Skeleton + Data Models + Core Traits |
| Phase 2 | ✅ | Account System (register/login/phone binding/identity verification) |
| Phase 3 | ✅ | Sealing Core (encryption + hash + certificate) |
| Phase 4 | ✅ | Unseal Engine (heartbeat/match/date/multi-confirm) |
| Phase 5 | ✅ | Secret Crush Scenario |
| Phase 6 | ✅ | Last Will Scenario |
| Phase 7 | ✅ | Time Capsule Scenario |
| Phase 8 | ✅ | Blockchain Timestamp (Merkle batch + MockChain + verify API) |
| Phase 9 | ✅ | OpenLink Integration (Identity Card + short link + credential verify) |
| Phase 10 | ✅ | OpenVault Integration (Shamir SSS M-of-N + VaultConnector) |
| Phase 11 | ✅ | Web Frontend API (CORS + JWT + WebSocket + OpenAPI spec) |
| Phase 12 | ✅ | Agent Action Protocol (agent.json + Action middleware + OpenMind) |

![Rust](https://img.shields.io/badge/Rust-1.75+-orange?logo=rust)
![Tests](https://img.shields.io/badge/tests-177%20%E2%9C%85-brightgreen)
![License](https://img.shields.io/badge/license-MIT-blue)
![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20macOS%20%7C%20Windows-informational)

## License

MIT
