# Phase 2 Plan: AppView Functional Implementation

**Phase:** phase-2-appview-functional
**Assessment:** `.kbd-orchestrator/phases/phase-2-appview-functional/assessment.md`
**Goals:** Make the AppView actually work — replace stubs, wire auth, add tests

---

## Change Breakdown

### c016-indexer-crud (P0)
**Goal:** Replace all `todo!()` stubs with actual DB writes
**Files:**
- `crates/appview-indexer/src/lib.rs` — implement `create_record`, `update_record`, `delete_record`
- `crates/appview-indexer/src/feed.rs` — post/like/repost CRUD
- `crates/appview-indexer/src/graph.rs` — follow/block/list/listitem CRUD
- `crates/appview-indexer/src/actor.rs` — profile CRUD

**Tasks:**
1. Implement `create_record` — parse CAR block → extract record → validate NSID → dispatch to collection handler
2. Implement `app.bsky.feed.post` create — parse text, reply refs, embeds → insert into `post` table
3. Implement `app.bsky.feed.like` create — insert into `like` table
4. Implement `app.bsky.feed.repost` create — insert into `repost` table
5. Implement `app.bsky.graph.follow` create — insert into `follow` table
6. Implement `app.bsky.graph.block` create — insert into `actor_block` table
7. Implement `app.bsky.actor.profile` create/update — upsert into `profile` table
8. Implement `app.bsky.graph.list` create/update — upsert into `list` table
9. Implement `app.bsky.graph.listitem` create — insert into `list_item` table
10. Implement `delete_record` for all collections — soft delete (set `deletedAt`) or hard delete
11. Handle idempotency — dedupe by `(uri, cid)` or use `ON CONFLICT`

**Acceptance:**
- Firehose events result in visible DB rows
- `cargo test -p appview-indexer` passes (add at least 1 test)

---

### c017-auth-viewer (P0)
**Goal:** Wire `Viewer`/`OptionalViewer` into all handlers
**Files:**
- `crates/appview-api/src/lib.rs` — add `OptionalViewer` to router state
- `crates/appview-api/src/graph.rs` — use `OptionalViewer` in get_blocks/get_mutes
- `crates/appview-api/src/notification.rs` — use `Viewer` in list_notifications/get_unread_count
- `crates/appview-auth/src/lib.rs` — ensure `OptionalViewer` works for unauthenticated requests

**Tasks:**
1. Change `get_blocks` signature to accept `OptionalViewer` — filter by `viewer_did` if present
2. Change `get_mutes` signature to accept `OptionalViewer` — filter by `viewer_did` if present
3. Change `list_notifications` to require `Viewer` — return 401 if missing
4. Change `get_unread_count` to require `Viewer` — return 401 if missing
5. Update `AppStateInner` to include auth configuration (JWT validation toggle)
6. Add `Authorization` header parsing to `decode_token`

**Acceptance:**
- `curl` without auth to `/xrpc/app.bsky.notification.listNotifications` returns 401
- `curl` with valid auth to `/xrpc/app.bsky.graph.getBlocks` returns only user's blocks
- Public endpoints (getProfile, searchActors) still work without auth

---

### c018-integration-tests (P1)
**Goal:** Add HTTP API integration tests
**Files:**
- `crates/appview-api/tests/integration_test.rs` — new file
- `crates/appview-db/src/lib.rs` — ensure test helpers available

**Tasks:**
1. Create test database setup helper (migrations + seed data)
2. Test `app.bsky.actor.getProfile` — valid actor → 200 with profile
3. Test `app.bsky.actor.searchActors` — search term → 200 with results
4. Test `app.bsky.graph.getFollows` — valid actor → 200 with follows list
5. Test auth: missing token on protected endpoint → 401
6. Test auth: invalid token → 401
7. Test auth: valid token → 200 with viewer-scoped results

**Acceptance:**
- `cargo test -p appview-api` passes with ≥5 tests
- Tests run in <10 seconds
- No test pollution (transactions rolled back)

---

### c019-prometheus-metrics (P1)
**Goal:** Add `/metrics` endpoint
**Files:**
- `crates/appview-api/src/lib.rs` — add metrics route on port 9090
- `crates/appview-api/src/metrics.rs` — new file with prometheus registry
- `crates/appview-bin/src/main.rs` — spawn metrics server

**Tasks:**
1. Add `metrics` feature flag or separate crate
2. Create `metrics::init()` — register histograms/gauges
3. Add Tower middleware for request latency tracking
4. Add DB pool gauge (active connections)
5. Add queue depth gauge (Fjall key count)
6. Add firehose cursor lag gauge
7. Expose on `:9090/metrics`

**Acceptance:**
- `curl localhost:9090/metrics` returns valid prometheus text
- Metrics update in real-time during requests

---

### c020-livekit-token (P2)
**Goal:** Implement LiveKit token minting endpoint
**Files:**
- `crates/appview-api/src/livekit.rs` — new file (or `tools_know_me.rs`)
- `crates/appview-livekit/src/token.rs` — implement `TokenMinter`

**Tasks:**
1. Add `POST /xrpc/tools.know-me.live.tokenMint` route
2. Parse request body (room name, participant identity)
3. Query `subscription` table for user's tier
4. Check tier allows room creation (Free=0, Creator=1, Pro=unlimited)
5. Generate LiveKit JWT with room grants
6. Return token + room URL

**Acceptance:**
- `curl` with valid auth returns JWT
- `curl` with Free tier requesting room returns 403 (or 402)

---

### c021-webrtc-signaling (P2)
**Goal:** Implement WHIP/WHEP signaling endpoints
**Files:**
- `crates/appview-api/src/webrtc.rs` — new file
- `crates/appview-webrtc/src/lib.rs` — session management

**Tasks:**
1. Add `POST /xrpc/tools.know-me.video.whip` — ingest SDP offer, store session
2. Add `POST /xrpc/tools.know-me.video.whep` — playback SDP offer, return answer
3. In-memory session store (HashMap<String, Session>)
4. Session cleanup after timeout

**Acceptance:**
- Endpoints accept valid SDP and return SDP answers
- Sessions expire after 1 hour of inactivity

---

## Execution Order

```
c016-indexer-crud (P0)
  → c017-auth-viewer (P0) [depends on indexer working for manual verification]
  → c018-integration-tests (P1) [depends on auth + indexer]
    → c019-prometheus-metrics (P1) [independent, can run in parallel]
    → c020-livekit-token (P2) [stretch, after core is solid]
    → c021-webrtc-signaling (P2) [stretch, after core is solid]
```

**Parallel tracks:**
- Track A: c016 → c017 → c018 (core functionality)
- Track B: c019 (metrics) — can start immediately, low dependency
- Track C: c020 → c021 (custom features) — start after Track A complete

---

## Rollback Plan

If any change introduces regressions:
1. `git stash` or `git checkout` to last known good commit
2. Re-run `cargo check` to confirm baseline
3. Re-apply change incrementally (smaller commits)
4. Run tests after each incremental step

---

## Success Criteria

Phase 2 is successful when:
1. Firehose events create visible DB records
2. Auth tokens control access to viewer-scoped endpoints
3. `cargo test` passes with ≥5 tests
4. `/metrics` returns valid prometheus data
5. Custom endpoints (LiveKit, WebRTC) return valid responses
6. All compilation warnings resolved or documented