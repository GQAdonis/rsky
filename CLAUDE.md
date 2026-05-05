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

## Intentional Divergence from Upstream `@atproto/pds`

rsky-pds is a Rust reimplementation of the AT Protocol PDS, not a port of the TypeScript reference implementation. Several design decisions diverge intentionally and permanently from upstream:

### Storage: Postgres-only (locked 2026-04-28)

Upstream `@atproto/pds` defaults to **SQLite for everything**: account-manager DB, sequencer DB, did-cache DB, and per-actor stores (one SQLite file per DID). rsky-pds uses **PostgreSQL for all of these**.

**Consequences:**
- rsky-pds is **not compatible** with the upstream `installer.sh` or `@atproto/pds` Docker image. Operators cannot swap one for the other without a full data migration.
- Per-actor isolation on Postgres is achieved via row-level discriminators (`actor_did` column on shared tables) with Postgres Row-Level Security policies — not via separate files.
- All parity claims against upstream are evaluated via a side-by-side **federation conformance harness** (`k8s/conformance/`) that proves protocol-level equivalence despite the storage divergence.

### Blob storage: S3-compatible object storage

Upstream uses on-disk blob storage under the data directory. rsky-pds uses an S3-compatible backend (tested with GCS HMAC keys). This makes horizontal scaling and cross-provider migration straightforward.

### Email: Mailgun / Resend

Upstream bundles SMTP configuration. rsky-pds uses Mailgun or Resend API keys (`RESEND_API_KEY` env var).

### What this means for contributors

- Do not attempt to add SQLite support to rsky-pds — this is out of scope by design.
- Protocol-level behavior (firehose output, `getRepo` CAR bytes, XRPC response shapes) MUST match upstream. Storage internals MUST NOT leak into wire format.
- When in doubt about expected protocol behavior, run the conformance harness.

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

## Memory Protocol (Surreal Memory MCP)

Before attempting any fix, search project memory:
```
mcp: search_memories({ query: "<symptom>", namespace: "rsky" })
```

After resolving any bug, log it immediately:
```
mcp: add_memory({
  content: "Fixed: <symptom>. Root cause: <cause>. Fix: <what changed>. Files: <paths>.",
  namespace: "rsky",
  tags: ["bug-fix", "<component>"]
})
```

This is mandatory. It prevents re-solving the same problems across sessions.

## Karpathy Continuous Wiki Protocol

After completing each change or phase:
1. Write a wiki entry to surreal-memory:
   ```
   add_memory({ content: "Wiki: <component>. What worked: ... What surprised: ... Watch for: ...", namespace: "rsky", tags: ["wiki", "<component>"] })
   ```
2. Update CLAUDE.md with any new "gotchas" discovered
3. Cross-reference from AGENTS.md

## Known Gotchas (maintained continuously)

| Issue | Root Cause | Fix | Logged |
|-------|-----------|-----|--------|
| relay CrashLoopBackOff (xShmMap) | GKE SSD PVC incompatible with SQLite WAL shared-memory | Delete PVC, relay re-syncs | prod-c001 |
| web client Server Components error | NEXTAUTH_URL missing from k8s configmap | Add to configmap.yaml, rollout restart | 2026-05-04 |
| Vercel analytics 404s | @vercel/analytics loaded on self-hosted deploy | Remove from layout.tsx, rebuild image | 2026-05-04 |
| appview 404 on XRPC routes | wintermute HTTPRoute (older creation time) won Gateway API precedence over appview route on same hostname | Delete wintermute gateway/httproute files from cluster and repo | prod-c002 |
| PDS kubectl apply causes ImagePullBackOff | statefulset.yaml contains IMAGE_TAG placeholder; direct apply uses literal string | Use kubectl patch for resource-only changes, never kubectl apply statefulset | 2026-05-05 |
| PDS OOMKilled/liveness fail under relay crawl | Rocket 0.5 default ~4 workers saturated by relay WS connections | Set ROCKET_WORKERS=32; relax probe (60s period, 15s timeout, 5 failures) | prod-c005 |
| appview wss:// forced despite ws:// config | appview-firehose hardcodes wss:// scheme regardless of input | Fix: preserve scheme from RELAY_HOSTS input (ws:// → ws://) | prod-c007 |
| appview InternalServerError on getTimeline | post_agg table missing from migration (profile_agg existed, post_agg did not) | Add post_agg to 001_initial_schema.sql; apply directly to Postgres | prod-c007 |
| deploy "no objects passed to apply" | k8s secret file deleted from repo but still referenced in deploy.yml | Recreate the secret file (minimal) or remove from deploy.yml | 2026-05-05 |
| appview relay connection timeout via gateway | Envoy Gateway 300s idle timeout kills long-lived WS; external URL adds gateway hop | Use internal cluster URL ws://rsky-relay:9000 instead of wss://relay.know-me.tools | prod-c007 |
| CI build jobs skipped despite code changes | path filter uses `contains(toJson(commits), '"rsky-appview/')` — fails when multiple commits pushed, relies on head_commit only | Use workflow_dispatch to force rebuild all images; push web-client submodule before dispatch | 2026-05-05 |
| web-client submodule unpushed commit breaks CI | Submodule pointer at SHA not on remote (SSH remote, commit only local) | Push submodule branch before triggering CI: `cd web-client && git push origin HEAD:main` | 2026-05-05 |
| relay doesn't auto-discover our PDS | rsky-relay uses RELAY_DISCOVERY_UPSTREAMS (upstream relays), not our own PDS | Call `POST /xrpc/com.atproto.sync.requestCrawl {"hostname":"pds.know-me.tools"}` explicitly | 2026-05-05 |
