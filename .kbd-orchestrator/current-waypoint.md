# KBD Waypoint — social.know-me.tools ATProto Stack

**Active Phase**: phase-2-commit-and-deploy (IN_PROGRESS)
**Queued Phase**: phase-3-pds-feature-parity (PLANNED, gated on phase-2 completion)
**Last updated**: 2026-04-28 by claude-code (kbd-plan phase-3)
**Phase status**: phase-2 IN_PROGRESS, phase-3 PLANNED

## Exact Next Command

```
/kbd-execute phase-2-commit-and-deploy
```

First change to execute:

```
openspec/changes/p2-c001/   ← START HERE (git submodule add)
```

## Execution Order

```
p2-c001  Add Ouranos git submodule        → claude-code  (automatable)
p2-c002  Commit phases 0+1 work           → claude-code  (automatable, needs p2-c001)
p2-c003  Seed GitHub Actions secrets      → HUMAN        (operational)
p2-c004  Bootstrap ArgoCD Application     → HUMAN        (operational, parallel with p2-c003)
p2-c005  Push to main — first CI run      → HUMAN        (operational, needs p2-c003 + p2-c004)
p2-c006  Configure Cloudflare DNS         → HUMAN        (operational, needs p2-c005)
p2-c007  Smoke test all endpoints         → HUMAN        (operational, needs p2-c006)
```

## Completed Phases Summary

| Phase | Status | Changes |
|-------|--------|---------|
| phase-0-foundation | COMPLETE | p0-c001..p0-c005 (Dockerfiles, k8s base, PostgreSQL, PDS manifests, CI) |
| phase-1-relay-feedgen | COMPLETE | p1-c001..p1-c006 (relay patch, initdb, relay+feedgen+labeler+jetstream k8s) |
| phase-2-commit-and-deploy | PLANNED | p2-c001..p2-c007 |

## Key Facts for Resuming

| Item | Value |
|------|-------|
| Git state | 7 modified + ~55 untracked — NOTHING COMMITTED YET |
| Submodule | `web-client/` is empty — must run p2-c001 first |
| Registry | `ghcr.io/know-me-tools` |
| Namespace | `atproto` |
| GKE cluster | `client-cluster`, `us-central1` |
| PVC StorageClass | `atproto-ssd-immediate` (`volumeBindingMode: Immediate`) |
| Secret pattern | All `secret.yaml` files use `${VAR}` envsubst — NO values in git |
| Labeler | Standby mode — `MOD_SERVICE_*` secrets can be `placeholder` initially |
| DNS | Cloudflare grey-cloud (DNS-only) required for cert issuance |
| Relay bind | Patched to `0.0.0.0` in `rsky-relay/src/server/server.rs` |
| Relay storage | WORKDIR=/data matches PVC mountPath — all CWD-relative paths go to PV |

## Queued: phase-3-pds-feature-parity

Phase 3 is fully planned. Do not begin execution until phase-2 finishes (p2-c003 → p2-c007 complete, smoke tests green).

**Reference baseline:** upstream `@atproto/pds@0.4.220` (commit `877e629`, 2026-04-24).
**Storage decision:** Postgres-only for everything (locked 2026-04-28). SQLite parity is out of scope.

**Phase-3 execution order (12 OpenSpec changes):**

```
p3-c001  Document Postgres-only divergence              → claude-code  (XS)
p3-c002  Low-effort sweep (5 unimplemented!() + did:web + uploads + getBlob + ozone proxy + requestCrawl)  → claude-code  (S)
p3-c003  used-refresh-token replay defense              → claude-code  (S)
p3-c004  Sequencer race fix + recovery hardening        → claude-code  (S)
p3-c005  rsky-lexicon refresh against upstream HEAD     → cursor       (M)
p3-c006  rsky-repo sync v1.1                            → claude-code  (M)  needs p3-c005
p3-c007  actor_store per-DID isolation hardening        → claude-code  (M)
p3-c008  Federation conformance harness                 → claude-code  (M)  needs p3-c004 + p3-c006 + p3-c007
p3-c009  OAuth provider core (PAR/authorize/token/JWKS) → claude-code  (XL) needs p3-c011
p3-c010  oauth-scopes Rust port                         → claude-code  (M)
p3-c011  Account-manager OAuth schema                   → claude-code  (M)  needs p3-c010
p3-c012  Wire OAuth into auth_verifier + pipethrough    → claude-code  (M)  needs p3-c009 + p3-c010 + p3-c011
```

**Phase-3 next command (after phase-2 done):**

```
/kbd-execute phase-3-pds-feature-parity
```

## OpenSpec health (post phase-3 plan)

- `openspec validate --all` → **34/34 passing** (30 changes + 4 capability specs)
- Capability specs: `pds-server` (new), `service-images`, `kubernetes-deployment`, `web-client`
- 18 pre-existing phases-0/1/2 changes were retrofitted into OpenSpec format on 2026-04-28
- OpenSpec tools configured: `claude-code`, `codex`, `opencode`, `windsurf`, `cursor`
