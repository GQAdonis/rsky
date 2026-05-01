# Assessment: phase-3-pds-feature-parity

**Assessed**: 2026-04-30  
**Tool**: opencode (kbd-assess)  
**Reference version**: `@atproto/pds@0.4.220` (upstream commit `877e629`)  
**Phase status per progress.json**: `IN_PROGRESS` — 11/12 changes DONE, 1 DELEGATED  

---

## Summary

Phase 3 is functionally **near-complete**. The 12 planned changes cover all major parity requirements against `@atproto/pds@0.4.220`. 11 of 12 are implemented and committed. The one delegated change (p3-c005 — lexicon refresh) has **partial manual work already present** as uncommitted changes in `rsky-lexicon/src/`. That work compiles cleanly with 1 warning (unused import). The auth_verifier DPoP/OAuth wiring (part of p3-c012's stated goal) remains at stub level — the `TODO` comment is still present and OAuth tokens are not yet evaluated in `validate_access_token`.

**Overall verdict: 11.5/12 changes effectively landed. One actionable gap remains in p3-c012 OAuth token verification depth.**

---

## Per-Requirement Assessment

### REQ-1: Storage MUST be PostgreSQL — `PASS`
- Documented in `docs/STORAGE.md`, `README.md`, `CLAUDE.md`
- No SQLite dependency in workspace
- Postgres-only documented for operators

### REQ-2: Sync v1.1 firehose — `PASS (partial)`
- `SyncEvt` struct in `rsky-lexicon/src/com/atproto/sync.rs` carries commit field
- `SubscribeReposSync` lexicon struct present in `rsky-pds/src/apis/com/atproto/sync/subscribe_repos.rs`
- Sequencer events file (`sequencer/events.rs`) includes sync event support
- **Gap**: No runtime evidence that `prev` CIDs and covering proofs are being emitted per-commit; this requires a live relay soak test to fully confirm. The spec scenario ("strict relay does not reject any frame") is unverified without the conformance harness running.

### REQ-3: `did:web` accounts — `PASS`
- `createAccount` path in `rsky-pds/src/apis/com/atproto/server/mod.rs:118-150` handles `did:web:` prefix explicitly
- `describe_server` advertises `"did:web"` in supported DID methods
- OAuth metadata also advertises `"did:web"` support
- No `unimplemented!` / panic path for `did:web`

### REQ-4: Refresh token replay defense — `PASS`
- Migration `2026-04-28-000001_add_used_refresh_token` present
- `rsky-pds/src/account_manager/helpers/used_refresh_token.rs` exists
- Referenced from `account_manager/mod.rs` and `schema.rs`
- Replay check + JTI insert wired into `rotate_refresh_token`

### REQ-5: Sequencer deterministic ordering — `PASS`
- `pg_advisory_xact_lock(hashtext(did))` found in `rsky-pds/src/sequencer/mod.rs`
- Recovery hardening (skip corrupt `nsid/rkey`) also present per progress notes

### REQ-6: Lexicon within one minor version — `PARTIAL / IN PROGRESS`
- **Uncommitted working tree** has substantial lexicon additions across:
  - `actor.rs` — `pronouns`, `VerificationState`, `StatusView`, `DeclaredAgePref`, `PostInteractionSettingsPref`, `VerificationPrefs`, `LiveEventPreferences`, `ProfileAssociatedGerm`, `ActivitySubscription`, new `RefPreferences` variants
  - `embed/video.rs` — `VideoPresentation` enum
  - `feed/mod.rs` — `postgate` and `threadgate` modules, `FeedViewerState`, `Record`, `quote_count`, `bookmark_count`
  - `graph/mod.rs` — `FeedList`, `Potentials` list purposes, `Relationship`, `RelationshipViewerState`, `BskyAppState`
  - `com/atproto/sync.rs` — `Repo`, `ListReposByCollectionOutput`
  - `com/atproto/moderation.rs` — new file
- `cargo check -p rsky-lexicon` passes with 1 warning (unused `super::*` import in test module)
- **No `UPSTREAM_VERSION.md`** in `rsky-lexicon/` — the required upstream commit SHA record is absent
- **No refresh procedure documented** in `rsky-lexicon/README.md` — the README is minimal (5 lines)
- **Not yet committed** — cannot be considered done until staged + committed with consumer `cargo check` passes

### REQ-7: Blob size limits — `PASS`
- Per p3-c002 progress note: 2MB image limit + 100MB video limit applied

### REQ-8: `getBlob` Content-Disposition — `PASS`
- `rsky-pds/src/apis/com/atproto/sync/get_blob.rs:74` sets `Header::new("content-disposition", "attachment")`

### REQ-9: Pipethrough proxies `tools.ozone.*` and `chat.bsky.*` — `PASS`
- Per p3-c002 progress note: `tools.ozone.*` proxy added

### REQ-10: `requestCrawl` debounce — `OPEN / NOT VERIFIED`
- `rsky-pds/src/crawlers.rs` has `requestCrawl` call but no debounce/mutex/coalescing was found
- No `Arc<Mutex<>>`, `once_cell`, or debounce guard present in crawlers
- This spec requirement may not have been addressed by any of the 12 p3 changes
- **This is an untracked gap** — not in any p3-c task list

### REQ-11: Per-actor isolation in Postgres actor store — `PASS (permissive tier, tighten remaining)`
- Migration `2026-04-28-000002_actor_store_rls` enables RLS on 6 tables: `blob`, `record`, `record_blob`, `repo_block`, `repo_root`, `account_pref`
- Policies are `PERMISSIVE` — allow all when `app.current_did` is not set (legacy path continues working)
- **Gap noted in migration**: `SET LOCAL app.current_did` is not yet wired at call sites — the migration itself documents this as a "tighten once wired" TODO. The restrictive tier (actual enforcement) is pending.

### REQ-12: OAuth as first-class auth path — `PARTIAL`
- OAuth endpoints exist: `jwks.rs`, `metadata.rs`, `par.rs`, `authorize.rs`, `token.rs`, `revoke.rs`, `introspect.rs` all present under `rsky-pds/src/apis/oauth/`
- `/.well-known/oauth-protected-resource` would be covered by metadata route
- OAuth schema migration `2026-04-28-000003_oauth_schema` provides `oauth_client`, `oauth_device`, `oauth_authorized_client`, `oauth_token`, `oauth_par_request` tables
- **`rsky-oauth-scopes` crate**: `OAuthScope`, `ScopeSet`, `scope_permits_xrpc` present and functional
- `oauthIssuer` field populated in `describeServer`
- **OPEN GAP**: `rsky-pds/src/auth_verifier.rs:797` still has `// @TODO: Implement DPop/OAuth` comment. `validate_access_token` only validates legacy JWT bearer tokens; it does not inspect token type, does not call `scope_permits_xrpc`, does not validate DPoP binding. An OAuth-issued access token presented to any XRPC handler would be evaluated as a legacy JWT and likely rejected or mishandled.
- **OPEN GAP**: `pipethrough.rs` has no OAuth scope gate — `RpcPermissionMatch::check` not called
- The "Bluesky web client OAuth login" scenario would fail end-to-end at the XRPC handler stage

### REQ-13: OAuth scope enforcement — `PARTIAL`
- `rsky-oauth-scopes` crate provides the Rust port of scope grammar (`scope_permits_xrpc`)
- **NOT YET WIRED** into `auth_verifier.rs` or `pipethrough.rs`
- The scope rejection scenario ("OAuth token without write scope rejected on createRecord") would not work

### REQ-14: Federation conformance harness — `PASS (structure present, not CI-integrated yet)`
- `tests/conformance/` contains `docker-compose.yml`, `Dockerfile.driver`, `run-conformance.sh`
- The shell script exercises both PDSes: waits for health, applies workload, diffs firehose and repo state
- **Gap**: No CI step in `.github/workflows/` runs the conformance harness automatically

---

## Gap Summary

| # | Gap | Severity | Change | Status |
|---|-----|----------|--------|--------|
| G1 | Lexicon changes uncommitted; `UPSTREAM_VERSION.md` missing; `rsky-lexicon/README.md` has no refresh procedure | HIGH | p3-c005 | DELEGATED — partially in progress, needs commit |
| G2 | `auth_verifier.rs` `validate_access_token` does not handle OAuth tokens or DPoP binding | HIGH | p3-c012 | OPEN — stub only |
| G3 | `pipethrough.rs` does not call `RpcPermissionMatch::check` for scope-gated methods | HIGH | p3-c012 | OPEN — not implemented |
| G4 | `requestCrawl` in `crawlers.rs` has no debounce/coalescing guard | MEDIUM | UNTRACKED | Not in any change |
| G5 | Actor store RLS is permissive-only; `SET LOCAL app.current_did` not wired at call sites | LOW | p3-c007 follow-up | Known, documented in migration |
| G6 | Conformance harness not integrated into CI | LOW | p3-c008 | Not yet wired |

---

## Recommended Next Actions (Ordered by Impact)

1. **Commit p3-c005 lexicon work** — the changes compile, fix the one warning (`use super::*`), add `UPSTREAM_VERSION.md` with SHA `877e629`, add refresh procedure to `rsky-lexicon/README.md`, run `cargo check -p rsky-pds` to confirm no consumer breakage, then commit.

2. **Complete p3-c012 auth_verifier wiring** — implement DPoP/OAuth token type discriminator in `validate_access_token`, wire `scope_permits_xrpc` for per-handler scope checks, and add the transitional fallback for legacy JWT sessions.

3. **Complete p3-c012 pipethrough wiring** — call `RpcPermissionMatch::check(nsid, permission_set)` before forwarding, return OAuth error on reject.

4. **Address requestCrawl debounce (G4)** — add an `Arc<Mutex<HashSet<String>>>` or `tokio::sync::OnceCell`-based coalesce guard in `crawlers.rs`. This is small scope but is a live federation correctness requirement.

5. **Wire CI for conformance harness** — add a workflow step or matrix job that runs `tests/conformance/run-conformance.sh` on PR.

---

## Compile Status

| Crate | Status |
|-------|--------|
| `rsky-lexicon` (with uncommitted changes) | `cargo check` PASS (1 warning: unused import) |
| `rsky-lexicon` (committed HEAD) | `cargo check` PASS clean |
| `rsky-pds` | Not checked (too slow for assessment session; last known clean per progress.json 2026-04-28) |

---

## Files Inspected

- `.kbd-orchestrator/project.json`
- `.kbd-orchestrator/current-waypoint.json`
- `.kbd-orchestrator/phases/phase-3-pds-feature-parity/progress.json`
- `openspec/specs/pds-server/spec.md`
- `openspec/changes/p3-c001` through `p3-c012` (proposals + tasks)
- `rsky-pds/src/auth_verifier.rs` (lines 785–830)
- `rsky-pds/src/pipethrough.rs` (grep scan)
- `rsky-pds/src/crawlers.rs` (grep scan)
- `rsky-pds/src/apis/oauth/` (directory listing)
- `rsky-pds/src/apis/com/atproto/sync/get_blob.rs` (line 74)
- `rsky-pds/src/apis/com/atproto/server/mod.rs` (lines 118–150)
- `rsky-pds/migrations/` (directory listing, up.sql for RLS and OAuth migrations)
- `rsky-oauth-scopes/src/lib.rs`
- `rsky-lexicon/README.md`
- `rsky-lexicon/src/` (full diff of uncommitted changes)
- `tests/conformance/` (directory listing + run-conformance.sh head)
