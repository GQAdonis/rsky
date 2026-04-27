# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What This Is

**rsky** is a full Rust implementation of the AT Protocol (Authenticated Transfer Protocol) — the decentralized social media protocol underlying Bluesky. It is maintained by Blacksky Algorithms and is pre-v1.0.0.

## Build and Test Commands

```bash
# Toolchain is pinned to Rust 1.86 (with clippy + rustfmt)
# via rust-toolchain.toml — rustup will auto-install it.

# Check a single crate
cargo check -p rsky-pds

# Build a single crate (release)
cargo build --release -p rsky-relay

# Test a single crate
cargo test --release -p rsky-pds

# Run a single test by name
cargo test --release -p rsky-repo -- merkle_search_tree

# Lint (format check, not applied)
cargo fmt -- --check

# Apply formatting
cargo fmt

# Run a service directly
cargo run -rp rsky-relay

# Satnav (Dioxus web UI) — requires `dx` CLI and Tailwind
dx serve
npx @tailwindcss/cli -i ./input.css -o ./assets/tailwind.css --watch
```

## Workspace Structure

The workspace has two categories of crates:

### Core Libraries (published on crates.io)

| Crate | Purpose |
|---|---|
| `rsky-syntax` | Parsers/validators for DIDs, Handles, NSIDs, AT URIs |
| `rsky-crypto` | secp256k1 and p256 signing, key serialization |
| `rsky-identity` | DID and handle resolution over DNS/HTTP |
| `rsky-common` | Shared utilities and data structures |
| `rsky-lexicon` | AT Protocol schema definitions and Bluesky API types |
| `rsky-repo` | Merkle Search Tree (MST) and repo serialization |
| `rsky-firehose` | Event stream WebSocket subscriber |

### Services and Applications (not published)

| Crate | Purpose | Storage |
|---|---|---|
| `rsky-pds` | Personal Data Server — Rocket web server, user repos | PostgreSQL + S3 |
| `rsky-relay` | High-throughput network relay, aggregates repo updates | SQLite + fjall |
| `rsky-wintermute` | Monolithic app-view indexer (ingester + backfiller + query) | heed (LMDB) |
| `rsky-feedgen` | Custom algorithmic feed generator | PostgreSQL |
| `rsky-labeler` | Firehose consumer for content moderation | — |
| `rsky-jetstream-subscriber` | Jetstream → JSON event transformer | — |
| `rsky-satnav` | CAR file browser web UI (Dioxus) | local CAR files |
| `rsky-pdsadmin` | Admin client for PDS management | — |
| `rsky-video` | Video processing service | — |

## Architecture

### Core Library Dependency Chain

```
rsky-syntax
    └── rsky-crypto
            └── rsky-identity
                    └── rsky-common
                            └── rsky-lexicon
                                    └── rsky-repo
                                            └── rsky-firehose
```

All services depend on `rsky-lexicon` and `rsky-common` at minimum.

### Network Data Flow

```
Users → rsky-pds (stores repos as MST)
          ↓
     rsky-relay (crawls and aggregates repo events from all PDSes)
          ↓
     rsky-wintermute (indexes relay data for query/search)
     rsky-feedgen (uses indexed data for custom feeds)
     rsky-labeler (monitors firehose for moderation)
```

### Key Data Structures

- **MST (Merkle Search Tree)**: Content-addressable, self-authenticating repo data structure in `rsky-repo`. Traversal uses `async-recursion`.
- **CBOR / DAG-CBOR**: Repo serialization via `serde_ipld_dagcbor`. CIDs are `lexicon_cid`.
- **CAR files**: Content-addressable archives for repo export/sync.

### Technology Choices by Service

- **rsky-pds**: Rocket 0.5 (synchronous), Diesel ORM, PostgreSQL, AWS S3, Mailgun
- **rsky-relay**: Tokio async, raw socket handling, SQLite (`rusqlite`), `fjall` for KV
- **rsky-feedgen**: Rocket + Diesel + PostgreSQL
- **rsky-wintermute**: Tokio, `heed` (LMDB bindings)
- **rsky-satnav**: Dioxus (Rust component framework targeting WASM + desktop)
- Crates using `Rust 2024` edition: `rsky-relay`, `rsky-wintermute` (others use 2021)

### Concurrency Patterns

- `DashMap` for concurrent in-memory KV stores
- `thingbuf` for lock-free ring buffers
- Tokio throughout; `magnetic` for message queues in relay

## Environment Setup

`rsky-pds` requires these environment variables (see CI for reference values):

```
PDS_HOSTNAME
PDS_SERVICE_DID
PDS_SERVICE_HANDLE_DOMAINS
PDS_ADMIN_PASS
PDS_JWT_KEY_K256_PRIVATE_KEY_HEX
PDS_PLC_ROTATION_KEY_K256_PRIVATE_KEY_HEX
PDS_REPO_SIGNING_KEY_K256_PRIVATE_KEY_HEX
PDS_MAILGUN_API_KEY
PDS_MAILGUN_DOMAIN
PDS_EMAIL_FROM_ADDRESS
PDS_EMAIL_FROM_NAME
```

External services required: PostgreSQL, S3-compatible storage, Mailgun.

## CI

GitHub Actions (`.github/workflows/rust.yml`) runs per-crate in parallel:
1. `cargo check -p <crate>` for every changed crate
2. `cargo build --release && cargo test --release` for changed crates
3. `cargo fmt -- --check` for formatting
4. Docker image build for services with Dockerfiles (`rsky-pds`, `rsky-relay`, `rsky-feedgen`, `rsky-labeler`, `rsky-jetstream-subscriber`)

Docker images are published to `ghcr.io/blacksky-algorithms`.

## Contribution Notes (from CONTRIBUTING.md)

- Do not submit large refactors or new external dependencies without discussion first.
- PRs should be scoped; separate library changes from service changes when practical.
- AT Protocol fundamentals are documented at `atproto.com` — understand PDS, Relay, and AppView concepts before making protocol changes.
