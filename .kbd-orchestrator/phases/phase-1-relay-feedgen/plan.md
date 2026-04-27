# Plan: phase-1-relay-feedgen
> Generated: 2026-04-27 | Tool: claude-code | Backend: openspec

## Phase Goal

Deploy rsky-relay, rsky-feedgen, rsky-labeler, rsky-jetstream-subscriber to GKE `atproto` namespace, wired together as a functioning ATProto relay + feed stack.

## Decisions Made (from user answers)

- **rsky-relay bind address**: Patch `0.0.0.0` directly in source (1-line change)
- **rsky-relay PVC**: 100Gi pd-ssd (expandable, generous start given fjall 320GiB ceiling)
- **rsky-labeler**: Deploy in standby mode — `ENABLE_CREATE_REPORT=false`, `ENABLE_CREATE_LABEL=false`, `ENABLE_CREATE_TAG=false`; placeholder secret values; activates when real credentials are added
- **PostgreSQL**: Add initdb ConfigMap with `rsky_feedgen` database creation

## Change Dependency Graph

```
p1-c001 (relay source patch + Dockerfile)
    ↓
p1-c002 (PostgreSQL initdb ConfigMap)        [parallel with p1-c003]
p1-c003 (rsky-relay k8s manifests)           [parallel with p1-c002; depends on p1-c001]
    ↓
p1-c004 (rsky-feedgen k8s manifests)         [depends on p1-c002]
    ↓
p1-c005 (rsky-labeler + rsky-jetstream-subscriber k8s manifests)  [depends on p1-c004]
    ↓
p1-c006 (CI updates: matrix + inject-secrets + README)  [depends on p1-c001..p1-c005]
```

## Ordered Change List

| # | Change ID | Title | Agent | Depends On | Parallel OK? |
|---|-----------|-------|-------|-----------|-------------|
| 1 | p1-c001 | Fix rsky-relay: bind address patch + Dockerfile | claude-code | — | No (blocks all relay work) |
| 2 | p1-c002 | PostgreSQL initdb ConfigMap for rsky_feedgen database | claude-code | — | Yes (parallel with p1-c003) |
| 3 | p1-c003 | rsky-relay k8s manifests | claude-code | p1-c001 | Yes (parallel with p1-c002) |
| 4 | p1-c004 | rsky-feedgen k8s manifests | claude-code | p1-c002 | No |
| 5 | p1-c005 | rsky-labeler + rsky-jetstream-subscriber k8s manifests | claude-code | p1-c004 | No |
| 6 | p1-c006 | CI workflow updates + README | claude-code | p1-c001..p1-c005 | No |

## Execution Notes

- **p1-c001** must complete first — the rsky-relay Dockerfile cannot exist until the bind address is patched
- **p1-c002** and **p1-c003** can execute in parallel after p1-c001
- **p1-c005** covers two services in one change (both are headless workers with minimal manifests)
- **p1-c006** is the integration change: adds rsky-relay to CI matrix, extends inject-secrets job for all new secrets, and updates k8s/README.md

## Definition of Done

- [ ] `rsky-relay/src/server/server.rs:157` binds to `0.0.0.0:9000`
- [ ] `rsky-relay/Dockerfile` exists and builds successfully
- [ ] `k8s/postgresql/initdb-configmap.yaml` creates `rsky_feedgen` database
- [ ] `k8s/rsky-relay/` — full stack including 100Gi PVC, Gateway, Certificate, HTTPRoutes
- [ ] `k8s/rsky-feedgen/` — Deployment, Service, Gateway (feed.know-me.tools), Certificate, HTTPRoutes
- [ ] `k8s/rsky-labeler/` — Deployment (standby mode), ConfigMap, Secret template
- [ ] `k8s/rsky-jetstream-subscriber/` — Deployment, ConfigMap, Secret template
- [ ] `.github/workflows/deploy.yaml` matrix includes rsky-relay (6 services total)
- [ ] inject-secrets job covers all new secrets
- [ ] `k8s/README.md` updated with new secrets
- [ ] All C-001 (no committed secrets), C-003 (envsubst templates), C-007 (Gateway+Certificate pairs) constraints satisfied
