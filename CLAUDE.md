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

## Rust Code Quality

### Validation Command (run before every commit)

```bash
# Root workspace
cargo clippy --workspace --all-targets --all-features --fix --allow-dirty

# rsky-appview sub-workspace
cd rsky-appview && cargo clippy --workspace --all-targets --all-features --fix --allow-dirty && cd ..

# rsky-pdsadmin sub-workspace
cd rsky-pdsadmin && cargo clippy --all-targets --all-features --fix --allow-dirty && cd ..
```

### Required Developer Tools

Install once:

```bash
cargo install cargo-geiger
cargo install cargo-modules
cargo install cargo-call-stack
cargo install cargo-udeps
cargo install cargo-nextest --locked
cargo install cargo-deny --locked
cargo install cargo-audit --locked
```

- **cargo-geiger**: Maps unsafe usage across the dependency tree
- **cargo-modules**: Visualizes module structure and call graph
- **cargo-call-stack**: Static call graph analysis
- **cargo-udeps**: Finds unused dependencies
- **cargo-nextest**: Faster, structured test runner (replaces `cargo test` in CI)
- **cargo-deny**: License, advisory, and ban policy enforcement
- **cargo-audit**: RustSec vulnerability database checks

### Using rust-skills for Standards-Compliant Code

Always invoke the `rust-skills` skill from actionbook before writing or reviewing Rust code:

```
Skill("rust-skills:m01-ownership")   # ownership/borrowing
Skill("rust-skills:m06-error-handling")  # error handling
Skill("rust-skills:m07-concurrency") # async/tokio patterns
```

Load the relevant skill for the module being changed and follow it.

### Audit Loop Pattern

Follow this cycle for all Rust changes:

```
Analyze → Patch → Re-run Clippy → Re-analyze → Benchmark → Continue
```

### Enterprise Audit Skill

A full enterprise audit prompt is available at `.claude/skills/rust-enterprise-audit.md`.
Load it when performing a deep review of any crate.

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
| appview loses firehose cursor on restart | Fjall queue path was `/tmp/appview-queue` (ephemeral); appview replays from seq=0 on every restart, never reaching live events | Changed to `/data/appview-queue` (PVC mount). Commit `3e92ebb`. Also configurable via `QUEUE_PATH` env var | 2026-05-05 |
| appview doesn't index our own PDS events despite relay having them | Relay has millions of public Bluesky events before our PDS events in global seq; appview replays all history before reaching our events | Seed actor/post rows directly via psql for immediate fix; then let appview catch up organically. `ensure_actor()` IS called for each commit event in indexer | 2026-05-05 |
| appview rejects ES256K JWT tokens from rsky-pds | jsonwebtoken crate's Algorithm enum doesn't include ES256K; header parse fails before insecure_disable_signature_validation() runs | Bypass jsonwebtoken::decode entirely: base64url-decode payload segment, deserialize Claims, validate exp only. Remove jsonwebtoken dep from appview-auth | prod-c015 |
| Ouranos login: Server Components crash on every page | app.bsky.actor.getPreferences returned 404 (not registered); Ouranos calls it on every render — Next.js Server Component error shown as generic digest error | Add get_preferences (empty prefs), put_preferences (200), getListFeed (empty feed) stubs to appview | prod-c016 |
| appview getAuthorFeed/getTimeline return record: {} — post text invisible | post_view_with_author() and post_view_plain() hardcoded empty serde_json::Map; DB PostRow.text field was never used | Build record from row.text and row.created_at: serde_json::json!({"$type":"app.bsky.feed.post","text":row.text,"createdAt":row.created_at}) | prod-c017 |
| rsky-feedgen crashloops on repost records | CreateRecord untagged enum only had Lexicon+Label variants; Lexicon internally-tagged enum only handled post/like/follow — repost panicked | Add AppBskyFeedRepost variant to Lexicon; add Unknown(serde_json::Value) catch-all to CreateRecord. Note: #[serde(other)] only works on unit enums, not internally-tagged | prod-c018 |
| notification.getUnreadCount 500 via PDS pipethrough | appview-auth Claims.sub was non-optional String; PDS service-auth JWTs use iss not sub — serde failed to deserialize, Viewer returned 401, pipethrough saw non-2xx → PDS returned 500 | Make Claims.sub and .iss both #[serde(default)]; copy iss→sub when sub empty | 2026-05-06 |
| feed_generator_agg table missing | migration 001_initial_schema.sql defined feed_generator but omitted feed_generator_agg (likeCount agg table) | Add table to migration + apply directly: kubectl exec postgresql-0 -- psql -d rsky -c "CREATE TABLE IF NOT EXISTS feed_generator_agg..." | 2026-05-06 |
| appview PDS firehose disconnects every 30s | IDLE_TIMEOUT=30s matched FIREHOSE_PING_INTERVAL=30s; last_message_time only reset on Binary frames, not Pong; low-traffic PDS had no events between pings | Increase IDLE_TIMEOUT to 90s; reset last_message_time on any received message (before Binary-only branch) | 2026-05-06 |
| appview getProfile returns {"profile":{...}} — client digest 3232795325 | GetProfileOutput struct wrapped ProfileViewDetailed in `profile` key; AT Protocol spec returns fields flat at root | Change GetProfileOutput to type alias: `pub type GetProfileOutput = ProfileViewDetailed`; handler returns `Json(profile)` | 2026-05-06 |
| appview misses our PDS events — RELAY_HOSTS only had relay | Appview subscribes to relay which replays millions of public events before reaching our PDS seq | Add wss://pds.know-me.tools to RELAY_HOSTS in k8s/rsky-appview/secret.yaml (comma-separated); appview spawns independent cursor per host | 2026-05-06 |
| kubectl secret patch overwritten by CI deploy | deploy.yml applies k8s manifests from repo, reverting any manual kubectl patch | Always edit the file in k8s/rsky-appview/secret.yaml and push to repo before patching cluster | 2026-05-06 |
| refreshSession panics with "invalid or out-of-range datetime" | from_micros_to_utc() in rsky-common/src/time.rs passed microseconds to NaiveDateTime::from_timestamp() which expects seconds — overflow panicked chrono | Replace with DateTime::from_timestamp_micros(micros).unwrap_or_else(\|\| DateTime::UNIX_EPOCH). Breaks NextAuth JWT_SESSION_ERROR → web client crash on login | 2026-05-06 |
| refreshSession returns ExpiredToken for brand-new tokens | store_refresh_token() in auth.rs called from_micros_to_utc((exp.as_millis() / 1000)) — passed seconds where micros expected; stored expiresAt as 1970-01-01T00:29:45Z | Change to from_millis_to_utc(exp.as_millis()) — exp.as_millis() is Unix ms timestamp, not a duration. Also delete corrupt rows: DELETE FROM pds.refresh_token WHERE "expiresAt" < '2025-01-01' | 2026-05-06 |
| appview firehose stuck at cursor forever — posts never indexed | relay sends CBOR message with negative integer in `op` header field; serde_ipld_dagcbor fails with "unexpected negative integer" deserializing into u8; error propagated via `?` so last_seq never advances; same message retried every 5 min indefinitely | Match read() result: treat Err as warn!+return Ok(()) instead of propagating. Connection stays alive, relay sends next seq naturally. File: rsky-appview/crates/appview-firehose/src/lib.rs | 2026-05-07 |
