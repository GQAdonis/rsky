# Phase Plan: phase-0-foundation

**Project**: social.know-me.tools ATProto Stack  
**Phase**: Phase 0 — Foundation & GitOps Scaffolding  
**Planned**: 2026-04-27  
**Change backend**: OpenSpec (`openspec/changes/`)  
**Execution agents**: claude-code (primary), cursor (parallel k8s manifests)

---

## Human Answers Incorporated

| Q | Answer | Impact |
|---|--------|--------|
| Q1 | GitHub org = `know-me-tools` | Registry: `ghcr.io/know-me-tools` |
| Q2 | Blob storage = GCS with HMAC keys | PDS secrets use `AWS_*` env vars pointing to `storage.googleapis.com` |
| Q3 | GKE SA already configured (same as conduit) | Reuse `GKE_SA_KEY` + `GHCR_PAT` secrets |
| Q4 | Plain StatefulSet with pgvector | Image: `pgvector/pgvector:pg17` |
| Q5 | DNS = Cloudflare | All DNS notes specify grey-cloud (DNS-only) for ACME cert issuance |

**PVC StorageClass**: `volumeBindingMode: Immediate` on all StatefulSet PVCs — required for GKE zonal clusters to avoid scheduling deadlock.

---

## Change Order

Changes are ordered by dependency. Execute sequentially; p0-c003 and p0-c004 can run in parallel.

```
p0-c001 → p0-c002 → p0-c003 ─┐
                               ├─→ p0-c005
               p0-c004 ───────┘
```

---

## Change 1: p0-c001 — Fix Dockerfiles

**File**: `openspec/changes/p0-c001/`  
**Agent**: claude-code  
**Blocks**: All image builds  
**Effort**: S (5 files, same pattern each)  

Replace `git clone https://github.com/blacksky-algorithms/rsky` in all Dockerfiles with `COPY . .` multi-stage builds. Runtime base: `debian:bookworm-slim`.

**Files changed**:
- `rsky-pds/Dockerfile`
- `rsky-feedgen/Dockerfile`
- `rsky-labeler/Dockerfile`
- `rsky-jetstream-subscriber/Dockerfile`
- `rsky-firehose/Dockerfile`

---

## Change 2: p0-c002 — PostgreSQL StatefulSet with pgvector

**File**: `openspec/changes/p0-c002/`  
**Agent**: claude-code  
**Depends on**: p0-c001 (namespace.yaml also created here)  
**Effort**: M  

Creates the `atproto` namespace and PostgreSQL StatefulSet.

**Key decisions**:
- Image: `pgvector/pgvector:pg17`
- StorageClass: `atproto-ssd-immediate` with `volumeBindingMode: Immediate` (mandatory for GKE)
- PVC: 20Gi pd-ssd, `reclaimPolicy: Retain`
- Secret: envsubst template for `${POSTGRES_USER}` and `${POSTGRES_PASSWORD}`

**Files created**:
- `k8s/namespace.yaml`
- `k8s/postgresql/storageclass.yaml`
- `k8s/postgresql/secret.yaml`
- `k8s/postgresql/pvc.yaml`
- `k8s/postgresql/statefulset.yaml`
- `k8s/postgresql/service.yaml`

---

## Change 3: p0-c003 — rsky-pds Kubernetes Manifests

**File**: `openspec/changes/p0-c003/`  
**Agent**: claude-code  
**Depends on**: p0-c001, p0-c002  
**Can parallel with**: p0-c004  
**Effort**: M  

Full StatefulSet + Gateway + Certificate + HTTPRoute manifests for rsky-pds at `pds.know-me.tools`.

**Key decisions**:
- Port 3000 via `ROCKET_PORT=3000`, `ROCKET_ADDRESS=0.0.0.0`
- GCS blob storage via `AWS_*` env vars → `https://storage.googleapis.com`
- PVC: 10Gi `atproto-ssd-immediate`
- Well-known endpoints served by rsky-pds itself (no nginx sidecar)
- Cloudflare DNS: grey cloud (DNS-only) required for ACME HTTP-01 challenge

**Files created**: 9 files in `k8s/rsky-pds/`

---

## Change 4: p0-c004 — Ouranos Web Client Manifests

**File**: `openspec/changes/p0-c004/`  
**Agent**: claude-code (or cursor in parallel)  
**Depends on**: p0-c001 (Dockerfile pattern)  
**Can parallel with**: p0-c003  
**Effort**: M  

Ouranos (Next.js) as git submodule + Deployment + Gateway + Certificate + HTTPRoute at `social.know-me.tools`.

**Key decisions**:
- Ouranos added as `web-client/` submodule
- `Dockerfile.web-client` at repo root (Next.js standalone output)
- Build args: `NEXT_PUBLIC_ATP_SERVICE_URL=https://pds.know-me.tools`
- AppView: `https://api.bsky.app` (Bluesky public, Phase 1)
- 2 replicas, no PVC, no secrets
- Cloudflare DNS: grey cloud initially, can proxy after first cert

**Files created**: `Dockerfile.web-client` + 7 files in `k8s/web-client/`

---

## Change 5: p0-c005 — Deploy Workflow + ArgoCD Application

**File**: `openspec/changes/p0-c005/`  
**Agent**: claude-code  
**Depends on**: p0-c002, p0-c003, p0-c004 (all manifests must exist)  
**Effort**: L  

GitHub Actions deploy workflow (GitOps, no direct kubectl) and ArgoCD Application manifest.

**Key decisions**:
- ArgoCD owns cluster state — CI only builds images and commits tag updates
- Matrix build strategy: 5 services in parallel
- `[skip ci]` commit message on image tag update (prevents loop)
- `envsubst` injects all secrets from GitHub Secrets
- `argocd app wait` for health verification
- Print Cloudflare DNS instructions at end

**Files created**:
- `k8s/argocd/application.yaml`
- `.github/workflows/deploy.yaml`
- `.github/scripts/update-image-tags.sh`
- `k8s/README.md` (secrets reference)

---

## Execution Assignment

| Change | Recommended Agent | Can Be Split? |
|--------|------------------|---------------|
| p0-c001 | claude-code | Yes — 5 independent Dockerfile edits |
| p0-c002 | claude-code | No — sequential, namespace first |
| p0-c003 | claude-code | Yes — manifests are independent files |
| p0-c004 | cursor or claude-code | Yes — parallel with p0-c003 |
| p0-c005 | claude-code | No — needs all manifests to reference |

---

## Definition of Done

Phase 0 is complete when:
- [ ] All 5 Dockerfiles build from local context
- [ ] `k8s/` directory fully populated and committed
- [ ] ArgoCD Application manifest committed to `k8s/argocd/`
- [ ] `.github/workflows/deploy.yaml` committed and passing
- [ ] First successful ArgoCD sync to `atproto` namespace on GKE
- [ ] `pds.know-me.tools` and `social.know-me.tools` resolve via Cloudflare DNS
- [ ] TLS certificates issued by Let's Encrypt on both domains
- [ ] rsky-pds `/xrpc/_health` returns 200
- [ ] Ouranos loads at `social.know-me.tools` and can authenticate to `pds.know-me.tools`
