# Assessment — phase-3-pds-feature-parity

**Goal of this phase:** Close the feature gap between `rsky-pds` (Rust, this repo) and the official `@atproto/pds` reference implementation so this PDS can serve as a drop-in replacement on the public AT Protocol network at parity with the version pinned in production deployments.

**Reference comparison points:**
- Local checkout of upstream source: `/Users/gqadonis/Projects/skytok-net/atproto/packages/pds` at version **`0.4.107`**.
- Production-pinned upstream image: `@atproto/pds@0.4.219` (per `/Users/gqadonis/Projects/travisjames/pds/service/package.json`). The `pds` directory the user pointed at is *not* source — it is the install/Docker wrapper that pulls the published npm package.
- Subject under test: `/Users/gqadonis/Projects/travisjames/rsky/rsky-pds` (forked from a pre-0.4.107 upstream snapshot, judging by surface and commits).

**Honesty calibration (sycophancy-correction applied):** The endpoint surface looks impressively close at file-level. That is misleading. The implementation has substantial behavioral, protocol, and operational gaps — particularly around OAuth, sync v1.1, did:web, blob storage flexibility, and the version drift to 0.4.219. This report leads with what is missing rather than what is present.

---

## Executive verdict

**Distance from parity: large, but tractable.** The Rust port is closer to a working PDS than to a finished one. It can plausibly run a single-tenant federated PDS for `did:plc` users on a Postgres + S3 stack against a static (pre-OAuth) client like older `@atproto/api` builds. It is **not** at parity with the PDS that today's Bluesky network and `@atproto/api ≥ 0.14.5` clients expect.

The three gaps that block parity, ranked:

1. **OAuth provider is entirely absent.** Upstream embeds `@atproto/oauth-provider` (introduced before 0.4.107, hardened through 0.4.219). `rsky-pds` has a single comment `// @TODO: Implement DPop/OAuth` in `src/auth_verifier.rs:788` and no OAuth tables, routes, DPoP handling, PAR, authorize, token, JWKS, or client metadata endpoints. This is the dominant blocker — modern Bluesky clients use OAuth flows; account sign-up via OAuth landed in 0.4.105.
2. **Sync v1.1 / firehose semantics are stale.** Upstream 0.4.101 added `prev` CIDs and covering proofs on `subscribeRepos`; 0.4.105 wrapped the new `#sync` event and deprecated `#handle`/`#tombstone`. `rsky-pds` only emits the older event shape. Relays that strictly enforce sync v1.1 will reject this PDS's firehose.
3. **Storage backend lock-in.** `rsky-pds` is hardwired to PostgreSQL + S3 (`actor_store/aws/s3.rs` is the only blob backend; account/sequencer stores are Diesel/Postgres). Upstream defaults to SQLite-per-actor + DiskBlobStore. The published install script and operator docs assume that default. This makes `rsky-pds` operationally non-substitutable for the standard self-host install — and makes per-actor isolation (one SQLite per repo, the upstream model) impossible.

The remaining gaps — `com.atproto.temp.*`, `did:web`, a few admin endpoints, version drift between 0.4.107 and 0.4.219, missing mailer templates — are real but bounded. With OAuth and sync v1.1 done, this is a 6–10 week effort for a single engineer; without them, calling this "compatible" is not honest.

---

## XRPC endpoint coverage

Generated from `find` on both trees. Names below are normalized to lexicon notation.

### `com.atproto.server.*`

| Endpoint | Upstream 0.4.107 | rsky-pds | Notes |
|---|---|---|---|
| createAccount | ✅ | ✅ | rsky version predates OAuth sign-up branch (PR #2945, 0.4.105). |
| createSession | ✅ | ✅ | Password + app-password only in rsky. |
| refreshSession | ✅ | ✅ | Rotates refresh token (`account_manager/mod.rs:256`). |
| deleteSession | ✅ | ✅ | |
| getSession | ✅ | ✅ | |
| describeServer | ✅ | ✅ | |
| createAppPassword / listAppPasswords / revokeAppPassword | ✅ | ✅ | rsky lacks the **privileged app password scope** added upstream (no `AuthScope::AppPassPrivileged`). |
| createInviteCode / createInviteCodes | ✅ | ✅ | |
| getAccountInviteCodes | ✅ | ✅ | |
| activateAccount / deactivateAccount | ✅ | ✅ | |
| checkAccountStatus | ✅ | ✅ | |
| requestAccountDelete / deleteAccount | ✅ | ✅ | |
| confirmEmail / requestEmailConfirmation / requestEmailUpdate / updateEmail | ✅ | ✅ | |
| requestPasswordReset / resetPassword | ✅ | ✅ | |
| reserveSigningKey | ✅ | ⚠️ **stub** | `src/apis/com/atproto/server/reserve_signing_key.rs:3` is literally `unimplemented!()`. |
| getServiceAuth | ✅ | ✅ | |

### `com.atproto.identity.*`

| Endpoint | Upstream | rsky | Notes |
|---|---|---|---|
| resolveHandle | ✅ | ✅ | |
| updateHandle | ✅ | ✅ | |
| getRecommendedDidCredentials | ✅ | ✅ | |
| requestPlcOperationSignature | ✅ | ✅ | |
| signPlcOperation | ✅ | ✅ | |
| submitPlcOperation | ✅ | ✅ | |
| **did:web support** | ✅ | ❌ | Hard `bail!("Not yet supporting did:web")` in `src/apis/com/atproto/server/mod.rs:115`. Affects account creation, DID doc validation, and any operator who wants `did:web` instead of `did:plc`. |

### `com.atproto.repo.*`

| Endpoint | Upstream | rsky | Notes |
|---|---|---|---|
| applyWrites / createRecord / putRecord / deleteRecord | ✅ | ✅ | |
| describeRepo / getRecord / listRecords | ✅ | ✅ | |
| listMissingBlobs | ✅ | ✅ | |
| uploadBlob | ✅ | ✅ | S3 only. |
| importRepo | ✅ | ✅ (202 lines) | Needs end-to-end migration test against upstream `goat account migrate` to confirm equivalence. |

### `com.atproto.sync.*`

| Endpoint | Upstream | rsky | Notes |
|---|---|---|---|
| getRepo / getLatestCommit / getRecord / getBlocks | ✅ | ✅ | |
| getBlob / listBlobs / listRepos | ✅ | ✅ | |
| getRepoStatus | ✅ | ✅ | |
| subscribeRepos | ✅ | ✅ (319 lines) | **Pre-sync-v1.1 event shape.** No `prev` CIDs on commits; emits `#handle`/`#tombstone` (deprecated upstream); does not emit `#sync` event on account creation. |
| listReposByCollection | ✅ (added post-0.4.107) | ❌ | Collections directory feature. |
| getCheckout / getHead (deprecated) | ✅ deprecated | ❌ | Acceptable to skip. |

### `com.atproto.admin.*`

| Endpoint | Upstream | rsky | Notes |
|---|---|---|---|
| deleteAccount | ✅ | ✅ | |
| disableAccountInvites / enableAccountInvites | ✅ | ✅ | |
| disableInviteCodes | ✅ | ✅ | |
| getAccountInfo / getAccountInfos | ✅ | ⚠️ partial | rsky has `get_account_info.rs` but not `get_account_infos.rs` (plural batch). |
| getInviteCodes | ✅ | ⚠️ stub paths | Two `unimplemented!()` on `get_invite_codes.rs:294` and `:327`. |
| getSubjectStatus / updateSubjectStatus | ✅ | ✅ | |
| sendEmail / updateAccountEmail / updateAccountHandle / updateAccountPassword | ✅ | ✅ | |

### `com.atproto.moderation.*`

| Endpoint | Upstream | rsky | Notes |
|---|---|---|---|
| createReport | ✅ | ⚠️ no handler file | No `create_report.rs` in `src/apis/com/atproto/moderation/`. Likely proxied — but worth verifying that the catchall in `pipethrough.rs` reaches Ozone correctly. |

### `com.atproto.temp.*`

| Endpoint | Upstream | rsky |
|---|---|---|
| checkSignupQueue | ✅ | ❌ — directory absent |

### `app.bsky.*` (locally implemented)

| Endpoint | Upstream | rsky |
|---|---|---|
| actor.getProfile / getProfiles / getPreferences / putPreferences | ✅ (read-after-write) | ✅ |
| feed.getActorLikes / getAuthorFeed / getFeed / getPostThread / getTimeline | ✅ (RAW) | ✅ |
| notification.registerPush | ✅ | ✅ |

Everything else (`app.bsky.*`, `chat.bsky.*`, `tools.ozone.*`) goes through pipethrough. rsky has the catchall (`src/apis/mod.rs:23` matches `app.bsky.` and `chat.bsky` prefixes). **`tools.ozone.*` is not in the catchall match** — likely a real gap for moderator tooling.

### Subscription endpoints not implemented

| Endpoint | Upstream | rsky |
|---|---|---|
| `com.atproto.label.subscribeLabels` | served by labelers, but PDS may proxy | ❌ — only present in `lexicon/lexicons.rs` codegen, no route handler |

---

## Subsystem-level gaps

### OAuth provider — **absent**

This is the largest single gap. Upstream pulls `@atproto/oauth-provider` and exposes:

- `/.well-known/oauth-protected-resource` (PDS-side) and `/.well-known/oauth-authorization-server` (entryway or self).
- PAR, authorize, token, JWKS, client-metadata endpoints (`auth-routes.ts`).
- DPoP-bound access tokens, replay protection, rotating refresh tokens with `used-refresh-token` table.
- Account sign-up *during* the OAuth flow (PR #2945, 0.4.105).
- Reset-password and PLC-operation flows hosted inside the OAuth UI (0.4.105).
- `account_manager/oauth-store.ts`, `device.ts`, `device-account.ts`, `authorization-request.ts`, `token.ts` — a substantial schema.

`rsky-pds` has none of this. It has password-only sessions (`create_session.rs`) and JWT-based app-password auth. Modern `@atproto/api` clients *can* still use createSession, so this is not immediately fatal — but it diverges from where the network is heading and from any client that has switched to OAuth-only.

**Effort estimate:** This is by far the largest item. There is no `@atproto/oauth-provider` Rust port that I'm aware of; it has to be written or the Rust PDS has to delegate to an external authorization server (entryway model). Either path is multi-week.

### Sync v1.1 — partial

`subscribe_repos.rs` emits the older event variants. Production relays in late 2024 / 2025 expect:
- `#sync` event on account creation (PR #3624, #3612).
- `prev` CIDs on each commit op + covering proofs (PR #3449).
- Deprecation of `#handle` and `#tombstone` in favor of `#identity` and `#account` (PR #3585).

rsky has `SubscribeReposIdentity` and `SubscribeReposSync` types in `rsky-lexicon` (`subscribe_repos.rs:13-19` imports them) — so the lexicon scaffolding is there — but commit-event emission still uses the legacy shape. **Verify by capturing rsky's firehose and diffing against an upstream PDS's firehose for the same write.** Until that diff is clean, federation parity is unproven.

### Account manager — close but incomplete

| Helper | Upstream (`packages/pds/src/account-manager/helpers/`) | rsky (`src/account_manager/helpers/`) |
|---|---|---|
| account.ts | ✅ | ✅ account.rs |
| password.ts (scrypt) | ✅ + scrypt.ts | ✅ password.rs |
| invite.ts | ✅ | ✅ invite.rs |
| repo.ts | ✅ | ✅ repo.rs |
| email-token.ts | ✅ | ✅ email_token.rs |
| auth.ts | ✅ | ✅ auth.rs |
| token.ts | ✅ | ❌ — refresh-token JWT logic likely inlined or partial |
| used-refresh-token.ts | ✅ | ❌ — reuse detection / replay defense missing |
| authorization-request.ts | ✅ (OAuth) | ❌ |
| device.ts / device-account.ts | ✅ (OAuth) | ❌ |

`used-refresh-token` is independently important: without it, refresh-token reuse cannot be detected, which is a real session-hijack mitigation upstream uses.

### Actor store — different storage model

Upstream:
- One **SQLite** database per actor (`actor-store/db/`).
- Reader/transactor split (`actor-store-reader.ts`, `actor-store-transactor.ts`, `actor-store-writer.ts`, `actor-store-resources.ts`).
- DiskBlobStore (default, 155 lines) and S3BlobStore via `@atproto/aws`.

rsky:
- Postgres with shared schema and `did` discriminators (`actor_store/repo/sql_repo.rs`).
- S3 only (`aws/s3.rs`).
- No reader/transactor abstraction — single mutable handle.

Implications:
1. Per-actor isolation upstream provides cheap repo export, copy, and migration. Postgres-shared is harder to migrate.
2. `DiskBlobStore` is the default for the install script. **A single-VPS user installing the upstream PDS gets disk blobs out of the box.** rsky requires an S3-compatible service. This is a deployment-shape divergence, not just a missing class.
3. There is a Bluesky-blessed pattern for moving repos between PDSes that depends on the per-actor SQLite layout (`importRepo`, `getRepo`, `activateAccount`). rsky's `importRepo` and `getRepo` exist, but are interacting with a fundamentally different storage shape — this is the highest-risk equivalence claim and needs an end-to-end migration test before it can be trusted.

### Mailer — likely incomplete templates

Upstream templates: `confirm-email.hbs`, `delete-account.hbs`, `plc-operation.hbs`, `reset-password.hbs`, `update-email.hbs`.
rsky `src/mailer/` has only `mod.rs` and `moderation.rs`. Template content is inlined — needs review against the 5 upstream templates to confirm parity of tokens, branding hooks, and the PLC-operation flow.

### PLC integration — present, narrow

`src/plc/` has `mod.rs`, `operations.rs`, `types.rs`. Signed PLC operations are wired (the rotation key is loaded in `apis/com/atproto/server/mod.rs:14`). What is *not* wired:
- `did:web` path (explicit `bail!`).
- The post-0.4.91 improved error reporting on PLC update failures (PR #4f2841efe).

### Sequencer — present, version-stale

`src/sequencer/{mod.rs, events.rs, outbox.rs}` (845 lines combined). Real implementation, not a stub. But:
- Event types reflect the pre-1.1 firehose (see Sync v1.1 above).
- Upstream PR #3580 fixed an out-of-order race when racing writes hit the same repo (0.4.104). Cannot tell from a quick read whether rsky has the equivalent fix or its own race.

### Pipethrough / proxy — present, scope-limited

`src/pipethrough.rs` is 603 lines — substantial. The dispatch test in `src/apis/mod.rs:23` matches `app.bsky.` and `chat.bsky` prefixes. It does **not** match `tools.ozone.*`. Upstream pipethrough proxies all unmatched lexicons. This is likely a real gap for moderator-facing endpoints.

### Background jobs

Upstream `background.ts` runs periodic work (token cleanup, etc.). I did not find an equivalent module in rsky. Worth a fresh search: refresh-token expiry cleanup, email-token cleanup, blob GC.

### Code-quality red flags

`grep` across `rsky-pds/src/` for `todo!()|unimplemented!()|TODO|FIXME|XXX|NotImplemented|unreachable!()|not yet`:
- **47 hits.**
- Hard fails (`unimplemented!()`/`todo!()`): `reserve_signing_key.rs:3`, `get_invite_codes.rs:294` and `:327`, `apis/com/atproto/server/mod.rs:160`, `db/mod.rs:25`. Five hard panic points.
- Documented stubs (`@TODO: Implement DPop/OAuth`): `auth_verifier.rs:788`.
- `bail!("Not yet supporting did:web")`: `apis/com/atproto/server/mod.rs:115`.

These are not catastrophic numbers, but each `unimplemented!()` is a runtime panic on an endpoint a real client *will* hit (especially `reserveSigningKey`, used during account migration).

### Recent commit activity (rsky-pds, last 25)

Last 25 commits on `main` are dominated by deployment/infra (image tags, ArgoCD config, secrets, image builds), the web client submodule, and a recent `disable invite codes, fix handle domain validation` (`e28f4f0`). **Zero commits in this window touch the gaps above.** That is consistent with the project being in deploy-finishing mode for phase-2 — it does not reflect a parity push.

---

## Version drift: 0.4.107 → 0.4.219

The local upstream checkout is at 0.4.107. Production pins 0.4.219. That is **112 patch releases** of drift, including (a partial selection from CHANGELOG):
- 0.4.108–0.4.130: ozone moderation surface expansion, OAuth provider hardening (multiple `oauth-provider` minor bumps).
- Continued sync v1.1 work and inductive firehose finalization.
- Lexicon updates that the rsky-lexicon codegen has not necessarily tracked.
- Likely security fixes; need to walk the changelog.

**Action:** when planning, treat 0.4.219 as the moving target, not 0.4.107. The local checkout is a stand-in for structure, not a parity reference. Pull the upstream 0.4.219 changelog window and inventory anything material.

---

## Risk-ranked gap list (input to phase-3 planning)

| # | Gap | Severity | Effort | Why it matters |
|---|-----|----------|--------|----------------|
| 1 | OAuth provider (PAR/authorize/token/JWKS/DPoP, sign-up flow, password reset flow, PLC-operation flow) | 🔴 Critical | XL (multi-week) | Network is converging on OAuth. Modern `@atproto/api` clients use it. |
| 2 | Sync v1.1 firehose semantics (`#sync` event, `prev` CIDs, covering proofs, deprecate `#handle`/`#tombstone`) | 🔴 Critical | M | Relays may reject this PDS's firehose. |
| 3 | Storage backend flexibility — DiskBlobStore + per-actor SQLite option | 🟠 High | L | Self-host operators expect the upstream storage shape. Repo migration tests depend on it. |
| 4 | `did:web` support | 🟠 High | S | Active `bail!` in account creation path. |
| 5 | `used-refresh-token` table + replay defense | 🟠 High | S | Real security improvement. |
| 6 | App-password privileged scope | 🟡 Medium | S | Upstream gates some endpoints on it. |
| 7 | `com.atproto.temp.checkSignupQueue` | 🟡 Medium | XS | Single endpoint; `temp` namespace. |
| 8 | `com.atproto.sync.listReposByCollection` | 🟡 Medium | S | Collections-directory feature. |
| 9 | `tools.ozone.*` pipethrough routing | 🟡 Medium | XS | Likely a one-line catchall fix in `apis/mod.rs:23`. |
| 10 | Admin: `getAccountInfos` (batch) and `getInviteCodes` (de-stub the two `unimplemented!()`) | 🟡 Medium | XS | Operator-facing. |
| 11 | `reserveSigningKey` (de-stub) | 🟡 Medium | XS | Used during account migration. |
| 12 | Mailer template parity (5 upstream templates vs unknown count in rsky) | 🟡 Medium | S | UX/branding equivalence. |
| 13 | Background jobs (token cleanup, email-token GC, blob GC) | 🟡 Medium | S | Long-running data hygiene. |
| 14 | Catch up 0.4.107 → 0.4.219 changelog (lexicon updates, ozone surface, fixes) | 🟠 High | M | The actual production target. |
| 15 | Subscribe-repos race fix (PR #3580 equivalent) | 🟡 Medium | S | Out-of-order events on concurrent writes. |
| 16 | End-to-end repo migration test (rsky ↔ upstream, both directions) | 🔴 Critical for trust | M | The main thing nobody has shown working. Until this passes, every claim above is unverified at the integration level. |

Effort legend: XS ≤ 1 day, S ≤ 1 week, M ≤ 2 weeks, L ≤ 1 month, XL > 1 month.

---

## What the planning phase needs to decide

These are the open architectural questions that should *not* be deferred into execution:

1. **OAuth strategy: implement in Rust, or run alongside an entryway?** Implementing `@atproto/oauth-provider` in Rust is a real R&D project. Running an entryway and pointing this PDS at it is faster but couples deployment.
2. **Storage strategy: keep Postgres-shared, or add SQLite-per-actor as an option?** This affects migration semantics and operational story. The current Postgres model is *not* wrong — it's just non-equivalent. Decide whether parity means feature-for-feature or shape-for-shape.
3. **Version target: 0.4.219, or follow tip-of-main?** Pinning to a moving target without an upgrade cadence is how this gap regrew. Pick one and own a bump policy.
4. **Verification model: what counts as "parity"?** Suggested: a federation conformance harness — spin up `rsky-pds` and an upstream `@atproto/pds@0.4.219` side-by-side, and run identical client traffic against both, diffing firehose output, account creation, repo migration, and OAuth flows. Without this, "parity" is a vibe, not a commit gate.

---

## Verification plan (for any future "we closed the gap" claim)

End-to-end checks that must pass before declaring parity:

1. **Firehose diff:** Same write sequence applied to rsky and upstream. Capture both `subscribeRepos` streams. Diff event shapes, including `prev`, `#sync`, and identity events. Must be byte-equivalent on lexicon-defined fields.
2. **Repo migration round-trip:** Create account on rsky → `getRepo` CAR → `importRepo` into upstream → `activateAccount` → write a record → `importRepo` back into rsky. Repo CID and MST root must match at each hop.
3. **OAuth flow against a real client:** Once OAuth lands, run the official Bluesky web client's OAuth login flow against rsky and confirm token issuance, refresh, and DPoP binding.
4. **`@atproto/api` smoke:** Run a fresh checkout of `@atproto/api` test suite (the parts that target a generic PDS) against rsky.
5. **Sync v1.1 conformance:** Run `goat` or an upstream relay against rsky's firehose for 24h and confirm zero rejection events.

---

## Resume context for executors

- **Active KBD waypoint at write-time:** `phase-2-commit-and-deploy` (deploy is mid-flight; do not confuse with this assessment).
- **This phase:** `phase-3-pds-feature-parity`, currently has **only this assessment**. No plan, no changes, no progress.json yet.
- **Next skill:** `kbd-plan phase-3-pds-feature-parity` — should produce an OpenSpec change set for items #1–#5 first, with the verification harness (item #16) as a parallel track.
- **Do not start execution** before phase-2 deploy lands. Pulling this much surface area while production is mid-migration is how regressions ship.

---

*Assessment produced 2026-04-28 by claude-code (sonnet-4.6 first pass; opus-4.7 finalization). Sycophancy-correction skill applied: leading with deltas, naming OAuth and sync v1.1 as critical rather than soft-pedalled, rejecting the "endpoint count looks close → close to parity" reading. Author: claude-code orchestrator on behalf of tjames@prometheusags.ai.*

---

# Refresh against 0.4.220 (2026-04-28)

The body above used a local upstream checkout pinned at `@atproto/pds@0.4.107`. That checkout was 113 patch releases stale. This section supersedes the version-drift commentary using a fresh sparse clone of `bluesky-social/atproto` at HEAD: commit **`877e629`** (2026-04-24), version **`0.4.220`**, one patch ahead of the production-pinned **`0.4.219`**. Source: `/tmp/atproto-latest/packages/pds`.

## Reference baseline (locked)

- **Upstream version:** `@atproto/pds@0.4.220`
- **Upstream commit:** `877e629`
- **`@atproto/oauth-provider`:** `0.16.1` (was `0.5.2` at 0.4.107 — 11 minor releases of evolution)
- **`@atproto/repo`:** `0.9.1` (was `0.7.1` at 0.4.107)
- **`@atproto/api`:** `0.19.x` (was `0.14.x` at 0.4.107)

## Findings the original section did not capture

### F1. Endpoint surface is identical at the file level since 0.4.107

`find packages/pds/src/api -name "*.ts"` at HEAD returns the same set of endpoints. **Zero new XRPC handlers in 113 patch releases.** The per-endpoint coverage matrix in the body above remains accurate. The gap is not in endpoint coverage — it is in the libraries each endpoint pulls in and the protocol semantics those libraries enforce.

### F2. OAuth provider drift is much larger than originally stated

`@atproto/oauth-scopes` is now a **separate, hard dependency** in `packages/pds/package.json` (granular scope grammar — `RpcPermissionMatch`, `ScopePermissions`, `PermissionSet`). It is referenced from `auth-verifier.ts:17`, `pipethrough.ts:11`, and `auth-output.ts:1`. Implementing OAuth in Rust now means implementing both the provider *and* the scope grammar.

New `account-manager` helpers introduced post-0.4.107, all part of the OAuth surface — none exist in rsky:

- `account-device.ts`
- `authorized-client.ts`
- `lexicon.ts` (lexicon-aware scope checks)
- `scope-reference-getter.ts`

OAuth is no longer a wrapper around basic auth — it is woven into the auth verifier and the proxy layer.

### F3. SQLite is upstream's first-class storage backend — we are intentionally diverging

Upstream `packages/pds/package.json` declares `better-sqlite3` as a top-level dependency, has a `test:sqlite-only` script, and supports SQLite for the **whole PDS** (account-manager DB, sequencer DB, did-cache DB, plus per-actor stores). Operators following the official self-host installer get SQLite by default for everything.

**Decision (2026-04-28, user-locked):** rsky-pds will not track SQLite parity. **Storage stays on PostgreSQL for everything** — account-manager, sequencer, did-cache, and the actor store. This is a deliberate operational divergence, not a gap to close.

Implications the planning phase must own:

- **`actor_store/` needs a Postgres-native shape that preserves per-actor isolation semantics.** Upstream gets isolation for free via one SQLite file per DID. On Postgres we either need per-DID schemas, hardened per-DID row-level discriminators with strict access guards, or per-DID logical databases. The current `sql_repo.rs` uses DID discriminators in shared tables — that needs hardening, not replacement.
- **`importRepo` / `getRepo` round-trip equivalence must be re-verified under the Postgres shape.** CAR bytes have to be byte-identical to upstream output for the same writes; only the storage *behind* getRepo changes.
- **README + CLAUDE.md must state Postgres-only explicitly** so operators don't expect drop-in compatibility with the upstream `installer.sh`.
- **The verification harness (item #16) is now non-optional**, because there's no upstream "Postgres-only PDS" to crib from. Side-by-side conformance is the only way to prove the storage divergence doesn't leak into protocol behavior.

### F4. Sync v1.1 is fully landed; `subscribeRepos.ts` is now a thin shim

Upstream `subscribeRepos.ts` is **73 lines** at HEAD. The protocol-correctness work moved into `@atproto/repo@0.9.x` and `@atproto/xrpc-server@0.10.x`. rsky's `subscribe_repos.rs` is 319 lines and reimplements the firehose locally on top of `rsky-repo` and `rsky-lexicon`. **The work to absorb sync v1.1 belongs in `rsky-repo` and `rsky-lexicon`, not in `rsky-pds`.** Specifically: covering proofs, `prev` CIDs, sync v1.1 wrapping, removal of deprecated sync fields (PR #2506).

### F5. Lexicon split into four upstream packages

`@atproto/lex-data`, `@atproto/lex`, `@atproto/lex-json`, `@atproto/lex-cbor` — lexicon handling was refactored into four packages. `rsky-lexicon` is monolithic. Wire-level invisible, but means rsky's lexicon codegen needs an explicit refresh cadence pegged to `lex-data` bumps.

### F6. Other items the original assessment missed

- **`@atproto/api` 0.14.x → 0.19.x.** Lexicon-derived types now explicitly include `$type`. Anything in rsky that consumes Bluesky API types must track this.
- **2 MB image upload limit** (PR #4823, 0.4.218). rsky enforces older limits.
- **100 MB video upload limit** (PR #3602, 0.4.105). Need to verify rsky's `uploadBlob` size cap.
- **`getBlob` forces browser download** via Content-Disposition (PR #4616, 0.4.209).
- **AppView response validation disabled** (PR #4797, 0.4.217). Operational change to mirror — avoids spurious validation failures on proxied responses.
- **Sequencer recovery script** ignores invalid commit paths (PR #4408, 0.4.217).
- **`requestCrawl` debounce** (PR #4408, 0.4.217). Avoids hammering relays.
- **Read-after-write skip for invalid records** (PR #4798, 0.4.217). Resilience change.
- **Sequencer race fix** (PR #3580, 0.4.104). Concurrent-write safety; original assessment named it but rsky equivalence is still unverified.
- **`InvalidCredentialsError` in PDS oauth store** (PR #4857, 0.4.220). Specific error-class wiring.

## Revised severity table (supersedes the body's table)

| # | Gap | Original | Revised | Why |
|---|-----|----------|---------|-----|
| 1 | OAuth provider | 🔴 Critical | 🔴 Critical (XL+) | Now also need `@atproto/oauth-scopes` Rust port; provider went 0.5.2 → 0.16.1. |
| 2 | Sync v1.1 firehose | 🔴 Critical | 🔴 Critical | Work moves into `rsky-repo` / `rsky-lexicon`. |
| 3 | ~~Storage flexibility~~ Postgres-only hardening | 🟠 High | 🟠 High (scope flipped) | Postgres-only locked. Work shifts to per-DID isolation hardening, divergence docs, and protocol-equivalence proof. |
| 4 | Lexicon data tracking | not listed | 🟠 High | `lex-data` bumps drive every protocol field change. |
| 5 | `@atproto/api` 0.14 → 0.19 type changes | not listed | 🟡 Medium | Affects pipethrough payload shapes. |
| 6 | Upload size limits (image 2MB, video 100MB) | not listed | 🟡 Medium | Config drift, easy fix, real user impact. |
| 7 | `getBlob` Content-Disposition | not listed | 🟢 Low | One-line. |
| 8 | Sequencer recovery + race fix (PR #3580, #4408) | partial | 🟠 High | Concurrent-write safety. |
| 9 | `requestCrawl` debounce | not listed | 🟢 Low | Avoids relay rate-limits. |
| 10 | `did:web` support | 🟠 High | 🟠 High | Unchanged. |
| 11 | `used-refresh-token` replay defense | 🟠 High | 🟠 High | Unchanged. |
| 12 | `tools.ozone.*` proxy | 🟡 Medium | 🟡 Medium | Unchanged. |
| 13 | Hard `unimplemented!()` cleanup | 🟡 Medium | 🟡 Medium | Unchanged. |
| 14 | Mailer template parity | 🟡 Medium | 🟡 Medium | Unchanged. |
| 15 | Background jobs | 🟡 Medium | 🟡 Medium | Unchanged. |
| 16 | E2E migration round-trip + federation conformance harness | 🔴 Critical for trust | 🔴 Critical for trust | Now non-optional under the Postgres-only divergence. |

## What this changes for planning

1. **Pin to 0.4.220 / commit `877e629`** as the reference baseline. Budget a recurring upstream-bump cadence; do not let drift regrow.
2. **OAuth is bigger than one item.** Split into: provider core, scope grammar (`@atproto/oauth-scopes`), account-device + authorized-client schema, lexicon-aware scope enforcement. Each is its own change in `kbd-plan`.
3. **Storage decision: Postgres-only, locked.** Parity here means "can run a PDS on the network and federate correctly," not "drop-in for the upstream installer." `kbd-plan` must produce sub-tracks for: per-DID isolation hardening on `actor_store/`, Postgres-native sequencer correctness (PR #3580 race fix equivalent), explicit Postgres-only callouts in README + CLAUDE.md, and the verification harness that proves protocol equivalence despite the storage divergence.
4. **Library-layer work dominates.** Material gap fractions live in `rsky-repo`, `rsky-lexicon`, `rsky-crypto`, `rsky-identity` — not in `rsky-pds`. The phase-3 plan needs explicit sub-tracks for those crates.
5. **Verification harness is the highest-leverage investment.** Without item #16, every parity claim above remains theoretical, and under Postgres-only divergence there is no upstream reference shape to copy.

## Next workflow stage

**`/kbd-plan phase-3-pds-feature-parity`** — consumes this refreshed assessment and emits OpenSpec change set for execution. Do not begin execution before phase-2-commit-and-deploy completes.

*Refresh authored 2026-04-28 by claude-code (opus-4.7) under explicit user direction to lock storage to Postgres-only and route the next stage through `/kbd-plan`. Reference: upstream `bluesky-social/atproto@877e629`.*
