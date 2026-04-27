# Assessment: phase-2-commit-and-deploy

**Date**: 2026-04-27  
**Assessed by**: claude-code  
**Phase goal**: Commit all phases 0+1 work, wire Ouranos submodule, push to main, first successful CI run, seed GitHub secrets, bootstrap ArgoCD, configure DNS, verify rollouts.

---

## Current State (from git status)

### Modified files (phases 0+1 source changes)
| File | Change |
|------|--------|
| `.gitignore` | Removed CLAUDE.md exclusion |
| `rsky-feedgen/Dockerfile` | Rewritten multi-stage with libpq |
| `rsky-firehose/Dockerfile` | Rewritten multi-stage without libpq |
| `rsky-jetstream-subscriber/Dockerfile` | Rewritten multi-stage without libpq |
| `rsky-labeler/Dockerfile` | Rewritten multi-stage with libpq |
| `rsky-pds/Dockerfile` | Rewritten multi-stage with libpq |
| `rsky-relay/src/server/server.rs` | Bind address patched `127.0.0.1` → `0.0.0.0` |

### Untracked files (phases 0+1 new additions)
| Path | Content |
|------|---------|
| `.github/scripts/update-image-tags.sh` | SHA tag update script |
| `.github/workflows/deploy.yaml` | Full CI/CD pipeline |
| `.kbd-orchestrator/` | KBD orchestration files |
| `CLAUDE.md` | Project guidance for Claude Code |
| `Dockerfile.web-client` | Ouranos Next.js container |
| `k8s/` | All Kubernetes manifests (55+ files) |
| `openspec/` | OpenSpec change management |
| `rsky-relay/Dockerfile` | New relay Dockerfile |

### Missing: web-client submodule
- `web-client/` directory is empty — no `.gitmodules` entry for `sudoWright/ouranos_atproto`
- `Dockerfile.web-client` references `COPY web-client/ .` — will fail without submodule
- CI workflow uses `actions/checkout@v4` with `submodules: recursive` — submodule must be registered

---

## Gaps Against Phase Goal

### G-001 — web-client git submodule not registered (BLOCKER)
`git submodule add https://github.com/sudoWright/ouranos_atproto web-client` has not been run.
Without it: Dockerfile.web-client COPY fails at build time; CI checkout does not clone Ouranos.

### G-002 — All phases 0+1 work uncommitted
7 modified + 15+ untracked paths. Nothing committed. CI will not trigger.

### G-003 — GitHub Actions secrets not seeded
Required before first CI run will succeed at inject-secrets step:
- `GKE_SA_KEY`, `GKE_PROJECT_ID`, `GHCR_PAT` (infrastructure)
- `POSTGRES_USER`, `POSTGRES_PASSWORD`
- `PDS_ADMIN_PASS`, 3× PDS key hex, `PDS_MAILGUN_API_KEY`, `PDS_MAILGUN_DOMAIN`
- `GCS_HMAC_ACCESS_KEY`, `GCS_HMAC_SECRET_KEY`, `GCS_BUCKET_NAME`
- `RELAY_ADMIN_PASSWORD`
- `RSKY_API_KEY`
- `MOD_SERVICE_DID`, `MOD_SERVICE_EMAIL`, `MOD_SERVICE_PASSWORD` (placeholder values acceptable)

### G-004 — ArgoCD Application not bootstrapped
`k8s/argocd/application.yaml` must be applied manually once before ArgoCD can sync.
The CI verify-sync job will fail until ArgoCD has the Application object.

### G-005 — Cloudflare DNS not configured
Four A records needed (DNS-only/grey-cloud for cert issuance):
- `pds.know-me.tools` → GKE Gateway IP (pds-gateway)
- `relay.know-me.tools` → GKE Gateway IP (relay-gateway)
- `feed.know-me.tools` → GKE Gateway IP (feedgen-gateway)
- `social.know-me.tools` → GKE Gateway IP (web-client-gateway)
Gateway IPs are only available after ArgoCD syncs and Envoy assigns external IPs.

### G-006 — rsky-relay Dockerfile liveness probe binary name
`pgrep -x rsky-relay` — confirm binary name matches the installed binary in Dockerfile.
Binary is installed as `rsky-relay` via `cargo install --path rsky-relay --root /usr/local`.
Cross-check: the `rsky-relay/Cargo.toml` `[[bin]]` name field.

### G-007 — GCS bucket creation not documented in runbook
`GCS_BUCKET_NAME` must reference an existing bucket with HMAC key configured.
This is an operational prerequisite outside GitOps; ensure bucket exists before CI run.

---

## What IS Ready

- All Dockerfiles present and structurally correct (multi-stage, workspace stub caching)
- All k8s manifests present: namespace, storageclass, postgresql, pds, relay, feedgen, labeler, jetstream-subscriber, web-client, argocd
- ArgoCD Application manifest committed (just needs one-time `kubectl apply`)
- CI/CD workflow covers build→tag→inject-secrets→verify-sync
- Secret templates use `${VAR}` envsubst pattern; no secrets in git
- rsky-relay bind address patched; WORKDIR=/data matches PVC mountPath
- initdb ConfigMap creates both `rsky` and `rsky_feedgen` databases with pgvector
- Labeler in standby mode (no credentials required to start)
- README.md documents all secrets, key generation, first-deploy steps

---

## Recommended Change Order

1. **p2-c001**: Add Ouranos git submodule (`git submodule add`)
2. **p2-c002**: Commit all phases 0+1 work in a single structured commit
3. **p2-c003**: Operational — Seed GitHub Actions secrets (human task, documented)
4. **p2-c004**: Operational — Bootstrap ArgoCD Application (human task, one `kubectl apply`)
5. **p2-c005**: Operational — First CI push, monitor build + inject-secrets
6. **p2-c006**: Operational — Get Gateway IPs, configure Cloudflare DNS, verify cert issuance
7. **p2-c007**: Operational — Smoke test endpoints (health checks, PDS, relay, feedgen, web client)

Changes p2-c001 and p2-c002 are automatable by Claude Code.
Changes p2-c003 through p2-c007 are human operational tasks with Claude Code providing runbook guidance.

---

## Risk Flags

| Risk | Severity | Mitigation |
|------|----------|------------|
| Ouranos Next.js build fails with unknown env vars | MEDIUM | Test `NEXT_PUBLIC_ATP_SERVICE_URL` accepted; Dockerfile uses `ARG` at build time |
| GKE cluster not running at first push | HIGH | Verify cluster is up before push; verify-sync timeout is 300s |
| cert-manager ACME challenge fails if DNS proxied (orange cloud) | HIGH | Cloudflare must be grey-cloud (DNS only) during cert issuance |
| rsky-relay liveness probe binary name mismatch | LOW | Verify `[[bin]]` name in Cargo.toml |
| `GCS_HMAC_ACCESS_KEY` missing at CI time | HIGH | Seed all secrets before first push |
