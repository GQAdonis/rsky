# Phase 2 Assessment: AppView Functional Implementation

**Phase:** phase-2-appview-functional
**Previous Phase:** phase-1-rsky-appview (REFLECTED)
**Goals:** Make the AppView actually work — replace stubs, wire auth, add tests

---

## Current State

### What's Working
- ✅ 13 crates compile cleanly
- ✅ 27 `app.bsky.*` XRPC endpoints return syntactically correct responses
- ✅ Firehose consumer connects to relay and enqueues jobs
- ✅ Indexer loop runs and dispatches jobs by collection type
- ✅ Database queries execute (sqlx + PostgreSQL)
- ✅ Deployment manifests ready (Dockerfile, k8s, GitHub Actions)

### What's Broken / Incomplete
- ❌ **Indexer writes nothing to DB** — All CRUD operations are `todo!()` stubs
- ❌ **No auth context** — `viewer_did` is hardcoded to `""` in all handlers
- ❌ **Graph blocks/mutes return all records** — No viewer filtering
- ❌ **Zero test coverage** — No tests exist for any crate
- ❌ **No metrics** — No `/metrics` endpoint for observability
- ❌ **Custom features not started** — LiveKit + WebRTC crates are empty shells

### Technical Debt from Phase 1
1. `appview-indexer/src/lib.rs` — 7 `todo!()` stubs for CRUD operations
2. `appview-api/src/graph.rs` — `get_blocks`/`get_mutes` pass `""` as viewer_did
3. `appview-api/src/notification.rs` — `viewer_did` hardcoded to `""`
4. `appview-auth/src/lib.rs` — `Viewer` extractor defined but unused in handlers
5. `appview-firehose/src/lib.rs` — Cursor save/load uses string-based key-value (not typed)

---

## Goals for Phase 2

### P0: Critical — AppView Must Function

**G1. Implement indexer CRUD operations**
- Replace all `todo!()` stubs with actual DB writes
- Collections: `app.bsky.feed.post`, `app.bsky.feed.like`, `app.bsky.feed.repost`, `app.bsky.graph.follow`, `app.bsky.graph.block`, `app.bsky.actor.profile`, `app.bsky.graph.list`, `app.bsky.graph.listitem`
- Each operation: parse record from CAR block → validate → insert/update/delete in DB
- Acceptance: Firehose events result in visible DB changes

**G2. Wire auth token extraction**
- Integrate `appview_auth::Viewer` into all handlers that need viewer context
- Update `graph::get_blocks`, `graph::get_mutes`, `notification::list_notifications`, `notification::get_unread_count`
- Acceptance: Requests with valid JWT return user-scoped results; requests without auth return 401 for protected endpoints

### P1: Important — Quality & Confidence

**G3. Add integration tests**
- HTTP API tests using `axum::Server` + `sqlx::test`
- Minimum 1 test per endpoint group (actor, feed, graph)
- Test auth flow: valid token → 200, invalid token → 401, missing token on protected endpoint → 401
- Acceptance: `cargo test` passes with ≥3 integration tests

**G4. Add Prometheus metrics**
- `/metrics` endpoint on separate port (9090)
- Metrics: request latency histogram, DB connection pool usage, queue depth, firehose cursor lag
- Acceptance: `curl localhost:9090/metrics` returns valid prometheus text format

### P2: Stretch — Custom Features

**G5. LiveKit token service (skeleton)**
- Implement `tools.know-me.live.tokenMint` endpoint
- Check subscription tier before minting (query `subscription` table)
- Return JWT with room grants
- Acceptance: `POST /xrpc/tools.know-me.live.tokenMint` returns valid LiveKit token

**G6. WebRTC signaling (skeleton)**
- WHIP ingest endpoint: `POST /xrpc/tools.know-me.video.whip`
- WHEP playback endpoint: `POST /xrpc/tools.know-me.video.whep`
- Session state stored in-memory (no persistence needed for skeleton)
- Acceptance: Endpoints accept valid SDP offers and return SDP answers

---

## Constraints

From `.kbd-orchestrator/constraints.md`:
- Never add dependencies without explicit discussion
- Never use `unwrap()`/`expect()` in non-test code
- Keep changes crate-scoped; avoid workspace-wide refactors
- Run `cargo fmt` before finishing edits
- Rust 1.86 pinned; do not upgrade toolchain
- sqlx dynamic queries only (no compile-time `query_as!` macro)

Additional phase-specific constraints:
- Indexer must handle duplicate events idempotently (firehose may re-deliver)
- Auth middleware must not break unauthenticated endpoints (getProfile, searchActors)
- Tests must use `#[sqlx::test]` with transaction rollback (no DB pollution)

---

## Risks

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| CAR block parsing complexity | Medium | High | Use existing `rsky-repo` crate for MST/CBOR parsing |
| Auth middleware breaks public endpoints | Medium | High | Make `OptionalViewer` the default; `Viewer` only for protected |
| sqlx test setup conflicts with workspace | Low | Medium | Use `sqlx::test` with `migrations` feature; ensure DB_URL env |
| LiveKit SDK API changes | Low | Medium | Pin `livekit-api` to exact version; wrap in trait for mocking |

---

## Resource Estimates

- **G1 (indexer CRUD)**: ~400 LOC, 2-3 sessions
- **G2 (auth wiring)**: ~100 LOC, 1 session
- **G3 (integration tests)**: ~300 LOC, 1-2 sessions
- **G4 (metrics)**: ~150 LOC, 1 session
- **G5 (LiveKit skeleton)**: ~200 LOC, 1 session
- **G6 (WebRTC skeleton)**: ~250 LOC, 1 session

**Total estimated**: ~1,400 LOC, 6-8 sessions

---

## Definition of Done

Phase 2 is complete when:
1. ✅ All indexer CRUD operations write to DB (firehose events persist)
2. ✅ Auth tokens are extracted and used for viewer-scoped queries
3. ✅ `cargo test` passes with ≥3 integration tests
4. ✅ `/metrics` endpoint returns prometheus metrics
5. ✅ `cargo fmt` + `cargo check` pass cleanly
6. ✅ Deployed to staging and manually verified (e.g., `curl appview.know-me.tools/xrpc/app.bsky.actor.getProfile?actor=did:plc:...`)