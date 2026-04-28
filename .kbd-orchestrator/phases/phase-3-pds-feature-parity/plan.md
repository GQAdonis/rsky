# Plan — phase-3-pds-feature-parity

## Goal

Close the feature gap between `rsky-pds` and the published `@atproto/pds@0.4.220` (commit `877e629`) so that this PDS can serve as a substitutable Personal Data Server on the public AT Protocol network — with the explicit constraint that storage stays on **PostgreSQL only** (no SQLite parity), per user decision 2026-04-28.

This plan reads from [.kbd-orchestrator/phases/phase-3-pds-feature-parity/assessment.md](.kbd-orchestrator/phases/phase-3-pds-feature-parity/assessment.md) and emits the changes below in OpenSpec format under `openspec/changes/p3-c00N/`. Each change has a `proposal.md`, `tasks.md`, and a `specs/<capability>/spec.md` delta that validates against `openspec validate`.

## Capabilities introduced

This phase introduces a new OpenSpec capability:

- **`pds-server`** — `openspec/specs/pds-server/spec.md` — the requirements that `rsky-pds` must satisfy to be a substitutable AT Protocol PDS. Phase-3 changes append `ADDED` and `MODIFIED` requirements to this capability.

It also touches the existing `kubernetes-deployment` capability when deployment configuration changes (e.g., env vars for new OAuth endpoints, secrets for upload-size limits).

## Ordered change list

Changes are ordered so that low-risk hardening lands first, library-layer work lands before the dependent server work, and OAuth lands last (largest item, depends on the schema and verifier groundwork). Execution should not begin until `phase-2-commit-and-deploy` finishes.

| # | ID | Title | Capability | Severity | Effort | Recommended agent | Depends on |
|---|----|-------|------------|----------|--------|-------------------|------------|
| 1 | p3-c001 | Document Postgres-only divergence in README + CLAUDE.md | `pds-server` | 🟢 Low | XS | claude-code | — |
| 2 | p3-c002 | Low-effort fixes: `unimplemented!()` cleanup, `did:web` support, upload limits, `getBlob` Content-Disposition, `tools.ozone.*` proxy, `requestCrawl` debounce | `pds-server` | 🟡 Mixed | S | claude-code | p3-c001 |
| 3 | p3-c003 | `used-refresh-token` replay defense | `pds-server` | 🟠 High | S | claude-code | — |
| 4 | p3-c004 | Sequencer race fix + recovery script hardening | `pds-server` | 🟠 High | S | claude-code | — |
| 5 | p3-c005 | rsky-lexicon refresh against upstream lexicons HEAD | `pds-server` | 🟠 High | M | cursor (parallel codegen sweep) | — |
| 6 | p3-c006 | rsky-repo: sync v1.1 (prev CIDs, covering proofs, `#sync` event) | `pds-server` | 🔴 Critical | M | claude-code | p3-c005 |
| 7 | p3-c007 | `actor_store` per-DID isolation hardening on Postgres | `pds-server` | 🟠 High | M | claude-code | — |
| 8 | p3-c008 | Federation conformance harness (rsky-pds vs `@atproto/pds@0.4.220` side-by-side) | `pds-server` | 🔴 Critical | M | claude-code | p3-c004, p3-c006, p3-c007 |
| 9 | p3-c009 | OAuth provider core: PAR / authorize / token / JWKS / DPoP | `pds-server` | 🔴 Critical | XL | claude-code (architectural) | p3-c011 |
| 10 | p3-c010 | `oauth-scopes` Rust port (`RpcPermissionMatch`, `ScopePermissions`, `PermissionSet`) | `pds-server` | 🟠 High | M | claude-code | — |
| 11 | p3-c011 | Account-manager OAuth schema (device, account-device, authorized-client, authorization-request, scope-reference-getter) | `pds-server` | 🟠 High | M | claude-code | p3-c010 |
| 12 | p3-c012 | Wire OAuth into `auth_verifier.rs` and `pipethrough.rs` with lexicon-aware scope checks | `pds-server` | 🔴 Critical | M | claude-code | p3-c009, p3-c010, p3-c011 |

## Out of scope (deferred)

- **Full SQLite parity** — explicitly rejected by user decision (2026-04-28). Postgres-only.
- **Full `@atproto/oauth-provider` 0.16.x feature mirror** in the first OAuth landing. Phase-3 lands the PAR/authorize/token/JWKS/DPoP core; advanced provider features (token revocation lists, OOB clients, custom scope grants) become phase-4 if needed.
- **`tools.ozone.*` proxying beyond catchall fix.** First landing is the one-line catchall fix in p3-c002. A full ozone-aware proxy is a separate phase.
- **Mailer template parity audit (15 templates).** Worth a dedicated phase-4 sweep; deferred so phase-3 can ship.
- **Background-jobs survey** (token cleanup, email-token GC, blob GC). Deferred to phase-4.

## Execution gates

1. **Do not start execution until `phase-2-commit-and-deploy` is fully complete** (web client deployed, smoke tests green, ArgoCD synced). Pulling phase-3 surface area while production is mid-migration risks regressions.
2. **The conformance harness (p3-c008) is non-optional.** Under Postgres-only divergence there is no upstream reference shape to copy; only side-by-side conformance proves protocol equivalence.
3. **OAuth (p3-c009 → p3-c012) is gated on the verification harness.** OAuth flows are not testable end-to-end without it.

## Verification

After this phase completes:

- `openspec validate --all` passes (existing 18 changes + 12 new ones + 4 capability specs).
- `cargo check --workspace` and `cargo test --workspace` pass.
- The conformance harness (p3-c008) reports byte-equivalent firehose output and byte-equivalent `getRepo` CAR output between rsky-pds and `@atproto/pds@0.4.220` for the canonical write sequence.
- The official Bluesky web client successfully completes an OAuth login flow against `pds.know-me.tools` and posts a record.
- A relay configured to enforce sync v1.1 strictly does not reject any frames from the rsky-pds firehose over a 24-hour soak.

## Next stage

After this plan is reviewed, run `/kbd-execute phase-3-pds-feature-parity` to walk the ordered change list. The `kbd-execute` skill picks the next pending change by reading `progress.json`, follows its `tasks.md`, and updates checkboxes / progress.json as it goes.
