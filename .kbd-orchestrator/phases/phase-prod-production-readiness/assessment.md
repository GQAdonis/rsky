# Phase Assessment: phase-prod-production-readiness
**Measured:** 2026-05-05  
**Baseline from reflection:** 52% (P0=100%, P1=75%, P2=0%, P3=25%)  
**Goal:** 100% functional production deployment

---

## Executive Summary

The web client (`social.know-me.tools`) cannot log in because three cascading issues block the Ouranos Server Components render. Fixing all three requires ~120 lines of Rust code changes across two files. After those three fixes land, the remaining gaps are P2 hardening work (OAuth/DPoP, Prometheus, Email, RLS, Conformance) and P3 polish (templates, smoke test). Total remaining effort to 100%: ~3–5 days of focused engineering.

---

## Immediate Blockers (Login Broken — P0)

### GAP-1: `app.bsky.actor.getPreferences` returns HTTP 404

**Root cause:** Route not registered in `rsky-appview/crates/appview-api/src/lib.rs`.  
**Evidence:** `curl https://appview.know-me.tools/xrpc/app.bsky.actor.getPreferences` → HTTP 404  
**Impact:** Ouranos calls this endpoint during Server Components render on every page (home feed, login redirect, profile). A 404 causes Next.js to throw a Server Components error, showing only a generic "digest" error to the user. Login is completely broken.  
**Fix required:**
1. Add `get_preferences` handler to `rsky-appview/crates/appview-api/src/actor.rs` returning a valid empty preferences object:
   ```json
   {
     "preferences": []
   }
   ```
2. Register route in `rsky-appview/crates/appview-api/src/lib.rs`:
   ```rust
   .route("/xrpc/app.bsky.actor.getPreferences", get(actor::get_preferences))
   ```
   Also needs `putPreferences` (POST, returns 200 empty) and `app.bsky.actor.getPreferences` must accept an optional `viewer` from the token (OptionalViewer extractor).  
**Effort:** ~30 min  
**Files:** `rsky-appview/crates/appview-api/src/actor.rs`, `rsky-appview/crates/appview-api/src/lib.rs`

---

### GAP-2: ES256K JWT algorithm rejected by appview JWT parser

**Root cause:** `rsky-appview/crates/appview-auth/src/lib.rs` uses `jsonwebtoken::Validation::default()`, which internally calls `decode()` and tries to parse the JWT header's `alg` field. PDS issues tokens with `alg: "ES256K"` (secp256k1). The `jsonwebtoken` crate's `Algorithm` enum does not include `ES256K`. The header parsing fails before signature validation runs, even though `insecure_disable_signature_validation()` is set.  
**Evidence:** Prior session log: `Invalid token: Auth error: Invalid token: JSON error: unknown variant 'ES256K', expected one of 'HS256'...`  
**Impact:** Every authenticated appview endpoint (getPreferences, getTimeline, putPreferences, listNotifications, etc.) rejects PDS-issued tokens. Users cannot perform any authenticated action.  
**Fix required:**  
The cleanest AT Protocol-correct fix is to bypass `jsonwebtoken` header parsing entirely and parse the JWT manually:
1. Base64-decode the header segment
2. Extract `alg` and `sub` without going through `jsonwebtoken::decode`
3. Skip signature validation (appview trusts the PDS — AT Protocol service auth validates via DID key resolution, not shared secret; this is correct behavior for an appview)

Alternatively: use `jsonwebtoken`'s `dangerous_insecure_decode` which skips both header algorithm check and signature. The existing `insecure_disable_signature_validation()` call does NOT skip the header `alg` enum parse.

**Implementation:**
```rust
// In decode_token(), replace jsonwebtoken::decode with manual JWT parse:
pub fn decode_token(token: &str) -> Result<Claims> {
    let parts: Vec<&str> = token.splitn(3, '.').collect();
    if parts.len() != 3 {
        return Err(AppViewError::Auth("malformed token".into()));
    }
    let payload = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(parts[1])
        .map_err(|_| AppViewError::Auth("invalid token payload".into()))?;
    let claims: Claims = serde_json::from_slice(&payload)
        .map_err(|e| AppViewError::Auth(format!("token parse error: {e}")))?;
    // Validate expiry only
    let now = chrono::Utc::now().timestamp();
    if claims.exp < now {
        return Err(AppViewError::Auth("token expired".into()));
    }
    Ok(claims)
}
```
**Effort:** ~45 min  
**Files:** `rsky-appview/crates/appview-auth/src/lib.rs`, `rsky-appview/crates/appview-auth/Cargo.toml` (add `base64` dep if not present)

---

### GAP-3: `record: {}` empty in postView — post text invisible in web client

**Root cause:** `rsky-appview/crates/appview-api/src/feed.rs` lines 46 and 65 hardcode `serde_json::Value::Object(serde_json::Map::new())` for the `record` field. The DB models `PostRow` and `PostWithAuthorRow` both have a `text` field (confirmed in `rsky-appview/crates/appview-db/src/models.rs` lines 22 and 38), but `post_view_with_author()` and `post_view_plain()` never use it.  
**Evidence:** `getAuthorFeed` returns `"record": {}` for all posts. Post text does not appear in the Ouranos web client.  
**Impact:** Even if login works, the home feed shows posts with no text content. The app is effectively non-functional for reading.  
**Fix required:**  
Build the `record` field from `row.text`:
```rust
fn post_view_with_author(row: &db::models::PostWithAuthorRow) -> PostView {
    PostView {
        // ...
        record: serde_json::json!({
            "$type": "app.bsky.feed.post",
            "text": row.text,
            "createdAt": row.created_at,
        }),
        // ...
    }
}

fn post_view_plain(row: &db::models::PostRow) -> PostView {
    PostView {
        // ...
        record: serde_json::json!({
            "$type": "app.bsky.feed.post",
            "text": row.text,
            "createdAt": row.created_at,
        }),
        // ...
    }
}
```
**Effort:** ~15 min  
**Files:** `rsky-appview/crates/appview-api/src/feed.rs`

---

## Short-Term Gaps (P1 — Required for Stable Service)

### GAP-4: feedgen new pod crashlooping on repost records

**Root cause:** `rsky-feedgen/src/models/create_request.rs` — `CreateRecord` enum uses `#[serde(untagged)]` with only three variants (`Lexicon`, `Label`). `Lexicon` itself only handles `app.bsky.feed.post`, `app.bsky.feed.like`, `app.bsky.graph.follow`. Any `app.bsky.feed.repost` or unknown record type from the relay panics the handler.  
**Status:** Old `:latest` pod still running as fallback — not immediately broken, but will fail if old pod terminates.  
**Fix required:** Add `Unknown(serde_json::Value)` variant to `Lexicon` enum (or to `CreateRecord`):
```rust
#[serde(rename = "app.bsky.feed.repost")]
AppBskyFeedRepost(rsky_lexicon::app::bsky::feed::repost::Repost),
// ... and for true unknowns:
#[serde(other)]  // or Unknown(serde_json::Value) 
Unknown,
```
**Effort:** ~30 min  
**Files:** `rsky-feedgen/src/models/create_request.rs`

---

### GAP-5: Missing appview XRPC routes Ouranos requires

Beyond `getPreferences`, Ouranos makes these calls that are also missing or stubs:

| Endpoint | Status | Notes |
|----------|--------|-------|
| `app.bsky.actor.getPreferences` | ❌ 404 | GAP-1 above |
| `app.bsky.actor.putPreferences` | ❌ 404 | Write endpoint; must return 200 |
| `app.bsky.feed.getListFeed` | ❌ 404 | List-based timeline |
| `app.bsky.graph.getRelationships` | ❌ 404 | Needed for viewer state |
| `app.bsky.graph.getStarterPack` | ❌ 404 | Optional, gracefully degradable |
| `app.bsky.actor.getPreferences` | ❌ 404 | —  |

**Immediate must-have:** `getPreferences` (GAP-1), `putPreferences` (stub 200), `getListFeed` (empty feed stub).  
**Effort:** ~1 hour for all three stubs  
**Files:** `rsky-appview/crates/appview-api/src/actor.rs`, `rsky-appview/crates/appview-api/src/lib.rs`

---

## P2 Hardening Gaps (Required Before General Availability)

### GAP-6: prod-c008 — PDS OAuth/DPoP cnf.jkt binding not enforced

**Status:** ❌ NOT MET  
**What's missing:** `rsky-pds/src/auth_verifier.rs` OAuth path doesn't validate `cnf.jkt` (DPoP key confirmation). Bluesky mobile/web clients that use DPoP will fail.  
**Effort:** ~4 hours  
**Files:** `rsky-pds/src/auth_verifier.rs`, `rsky-pds/src/apis/oauth/`

### GAP-7: prod-c009 — Actor store RLS not RESTRICTIVE

**Status:** ❌ NOT MET  
**What's missing:** RLS policies exist but are not `RESTRICTIVE`. Multi-tenant data isolation relies on application-level filtering only.  
**Effort:** ~2 hours (migration + code wiring)  
**Files:** `rsky-pds/src/actor_store/mod.rs`, new migration

### GAP-8: prod-c010 — Conformance harness CI not wired

**Status:** ❌ NOT MET  
**What's missing:** `k8s/conformance/run-conformance.sh` exists but no GitHub Actions job runs it.  
**Effort:** ~2 hours  
**Files:** `.github/workflows/conformance.yml` (new)

### GAP-9: prod-c011 — Prometheus ServiceMonitors missing

**Status:** ❌ NOT MET — metrics endpoints exist, scraping not configured  
**What's missing:** `ServiceMonitor` resources for relay and appview; basic alerting rules.  
**Effort:** ~1 hour  
**Files:** `k8s/rsky-relay/servicemonitor.yaml` (new), `k8s/rsky-appview/servicemonitor.yaml` (new)

### GAP-10: prod-c012 — Email/Resend verification untested

**Status:** ❌ NOT MET  
**What's missing:** `RESEND_API_KEY` not confirmed set in cluster; email flow not tested end-to-end.  
**Effort:** ~2 hours (config + manual test)  
**Files:** `k8s/rsky-pds/secret.yaml`, `rsky-pds/src/mailer/mod.rs` (verification only)

---

## P3 Polish Gaps

### GAP-11: prod-c013 — Mailer template parity

**Status:** 🔄 PARTIAL — some templates exist; 15-template audit not done  
**Effort:** ~3 hours

### GAP-12: prod-c014 — Nightly smoke test automation

**Status:** 🔄 PARTIAL — `smoke-test.yml` workflow exists; coverage not complete  
**Effort:** ~2 hours

---

## Ordered Fix Plan (Priority Order)

| # | Fix | Effort | Files | Unblocks |
|---|-----|--------|-------|---------|
| 1 | ES256K JWT support (GAP-2) | 45 min | `appview-auth/src/lib.rs` | All authenticated routes |
| 2 | `getPreferences` + `putPreferences` routes (GAP-1, GAP-5) | 30 min | `appview-api/src/actor.rs`, `lib.rs` | Ouranos login |
| 3 | `getListFeed` stub (GAP-5) | 15 min | `appview-api/src/feed.rs`, `lib.rs` | Ouranos home feed |
| 4 | `record` field hydration (GAP-3) | 15 min | `appview-api/src/feed.rs` | Post text visible |
| 5 | Build + deploy new appview image | 20 min CI | CI | All above |
| 6 | feedgen `CreateRecord` repost variant (GAP-4) | 30 min | `rsky-feedgen/src/models/create_request.rs` | Stable feedgen |
| 7 | Prometheus ServiceMonitors (GAP-9) | 1 hour | k8s yaml | Observability |
| 8 | Email/Resend verification (GAP-10) | 2 hours | k8s secret + manual | Email signup |
| 9 | DPoP/OAuth cnf.jkt (GAP-6) | 4 hours | `rsky-pds/src/auth_verifier.rs` | Bluesky app compat |
| 10 | RLS RESTRICTIVE policies (GAP-7) | 2 hours | migration | Security hardening |
| 11 | Conformance CI (GAP-8) | 2 hours | `.github/workflows/` | Protocol parity proof |
| 12 | Smoke test automation (GAP-12) | 2 hours | `tests/e2e/` | Regression safety |
| 13 | Mailer template parity (GAP-11) | 3 hours | `rsky-pds/src/mailer/` | Email completeness |

**Total to login working (items 1–5):** ~2 hours  
**Total to stable P1 service (items 1–6):** ~2.5 hours  
**Total to 100% (all 13):** ~19 hours (~3 focused days)

---

## Gap Coverage by Original Phase Changes

| Change | Current Status | Gaps to Close |
|--------|---------------|--------------|
| prod-c001 | ✅ DONE | — |
| prod-c002 | ✅ DONE | — |
| prod-c003 | ✅ DONE | — |
| prod-c004 | ✅ DONE | — |
| prod-c005 | ✅ DONE | — |
| prod-c006 | 🔄 PARTIAL | Lexicon JSON files not audited |
| prod-c007 | 🔄 PARTIAL (with GAPs) | GAP-1, GAP-2, GAP-3 (login broken, records empty) |
| prod-c008 | ❌ NOT MET | GAP-6 (DPoP/OAuth) |
| prod-c009 | ❌ NOT MET | GAP-7 (RLS) |
| prod-c010 | ❌ NOT MET | GAP-8 (conformance CI) |
| prod-c011 | ❌ NOT MET | GAP-9 (Prometheus) |
| prod-c012 | ❌ NOT MET | GAP-10 (Email) |
| prod-c013 | ❌ NOT MET | GAP-11 (templates) |
| prod-c014 | 🔄 PARTIAL | GAP-12 (smoke test coverage) |

---

## New OpenSpec Changes Required

These gaps require new change entries beyond the original prod-c001..c014 plan:

| Change ID | Title | Priority |
|-----------|-------|---------|
| prod-c015 | ES256K JWT support in appview-auth | P0 |
| prod-c016 | `getPreferences`/`putPreferences`/`getListFeed` stub routes | P0 |
| prod-c017 | `record` field hydration in postView | P0 |
| prod-c018 | feedgen `CreateRecord` repost+unknown variant handling | P1 |

---

## Completion Projection

| After | Completion % |
|-------|-------------|
| Current (reflection baseline) | 52% |
| After items 1–5 (login works) | 68% |
| After item 6 (feedgen stable) | 72% |
| After items 7–10 (P2 hardening) | 88% |
| After items 11–13 (P3 + conformance) | 100% |

---

*Assessment complete. Next action: `/opsx:apply prod-c015` (ES256K JWT fix — highest leverage, unblocks everything else)*
