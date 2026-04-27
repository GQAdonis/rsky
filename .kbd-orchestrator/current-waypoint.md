# KBD Waypoint — social.know-me.tools ATProto Stack

**Active Phase**: phase-2-commit-and-deploy  
**Last updated**: 2026-04-27 by claude-code (kbd-plan)  
**Phase status**: PLANNED  

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
