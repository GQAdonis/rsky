# Phase Assessment: phase-0-foundation

**Project**: social.know-me.tools ATProto Stack  
**Phase**: Phase 0 — Foundation & GitOps Scaffolding  
**Assessed**: 2026-04-27  
**Assessed by**: claude-code (kbd-assess)  
**Status**: FIRST RUN — no prior progress

---

## Phase Goal

Establish all scaffolding required for every subsequent phase to build on:
- Kubernetes namespace and ArgoCD Application wired to this repo
- GitHub Actions build + image-tag-commit workflow (GitOps pattern)
- k8s directory structure mirroring conduit's layout
- PostgreSQL StatefulSet (shared by rsky-pds and rsky-feedgen)
- Web client decision finalized and documented
- OpenSpec and KBD orchestration live in main branch

---

## What Exists Today

### Codebase (rsky repo)

| Item | Status |
|------|--------|
| 17 Rust workspace crates | ✅ Present |
| Dockerfiles: rsky-pds, rsky-feedgen, rsky-labeler, rsky-jetstream-subscriber, rsky-firehose | ✅ Present (need updates) |
| `.github/workflows/rust.yml` — per-crate CI build/test | ✅ Present |
| `.github/workflows/rsky-pdsadmin.yml` | ✅ Present |
| `.kbd-orchestrator/` directory | ✅ Just created |
| `openspec/specs/` — 4 spec files | ✅ Just created |
| `CLAUDE.md` | ✅ Just created |
| `k8s/` directory | ❌ Missing |
| `.github/workflows/deploy.yaml` | ❌ Missing |
| ArgoCD Application manifest | ❌ Missing |
| PostgreSQL k8s manifests | ❌ Missing |
| rsky-pds k8s manifests | ❌ Missing |
| Web client k8s manifests | ❌ Missing |
| Any other service k8s manifests | ❌ Missing |

### Cluster (GKE client-cluster, us-central1)

| Item | Status |
|------|--------|
| Envoy Gateway (`gatewayClassName: eg`) | ✅ Running (proven by conduit) |
| cert-manager + ClusterIssuer `letsencrypt` | ✅ Running (proven by conduit) |
| ArgoCD | ✅ Available on cluster |
| `atproto` namespace | ❌ Not yet created |
| GHCR pull secret in `atproto` namespace | ❌ Not yet created |

### Existing Dockerfiles (need registry update)

All existing Dockerfiles reference `ghcr.io/blacksky-algorithms` as the upstream or build from the upstream git repo. For our deployment we need images built from **this fork** and pushed to `ghcr.io/know-me-tools`.

The `rsky-pds` Dockerfile clones from `github.com/blacksky-algorithms/rsky` at build time — this must be changed to build from the local context.

---

## Gap Analysis Against OpenSpec

### spec 00 (Project Overview) — Phase 0 goals

| Goal | Gap |
|------|-----|
| GitOps deploy via ArgoCD | No `k8s/` dir, no ArgoCD Application, no deploy workflow |
| `atproto` namespace | Not created |
| All tools can read OpenSpec | ✅ Specs created; needs git commit |
| KBD waypoint accessible to all tools | ✅ Waypoint written |

### spec 01 (Web Client) — Decision needed for Phase 0

**Research Findings:**

1. **Bluesky `social-app`** (React Native Web/Expo) — most features, but build complexity is high (Expo managed workflow, not a simple `docker build`). Static web export is possible but requires EAS build or local Expo CLI. Configuring a default PDS requires setting `EXPO_PUBLIC_ENV=production` and patching the service config — not a clean env-var-only config.

2. **Ouranos** (`github.com/sudoWright/ouranos_atproto`) — Next.js app, friendly Bluesky web client. Deploys to Vercel natively but is a standard Next.js app — `docker build` works, `NEXT_PUBLIC_*` env vars configure AppView/PDS endpoints. **Best candidate for self-hosted Docker deployment.**

3. **Langit** (`codeberg.org/intrnl/langit`) — opinionated SvelteKit PWA, Chrome/Firefox only, no Safari. Very lightweight. Good for power users, less suitable as a default social client.

4. **Blacksky's own AppView** — Blacksky runs `api.blacksky.community` as a production AppView powered by rsky-wintermute. This is the proven path for a fully sovereign stack in Phase 3+.

**Web Client Decision (Phase 0/1):**

> **Recommend: Ouranos** over social-app for Phase 1 self-hosted deployment.
>
> Rationale:
> - Standard Next.js → clean `docker build` → straightforward k8s Deployment
> - `NEXT_PUBLIC_ATP_SERVICE_URL` and `NEXT_PUBLIC_ATP_APPVIEW_URL` env vars configure PDS + AppView
> - Works with Bluesky AppView for network reads (hybrid mode)
> - No Expo/EAS build complexity
> - Actively maintained, MIT license
>
> social-app remains the UX gold standard but is operationally harder to self-host as a Docker container. Revisit when Expo web export matures or when we have bandwidth for the build pipeline.

**Spec 01 update required:** Change recommended client from social-app to Ouranos.

### spec 02 (Infrastructure) — Fully specified, nothing built

| Component | Spec Status | Built Status |
|-----------|-------------|--------------|
| Namespace manifest | ✅ Spec complete | ❌ Not built |
| ArgoCD Application | ✅ Spec complete | ❌ Not built |
| GitHub Actions deploy workflow | ✅ Pattern from conduit | ❌ Not built |
| rsky-pds StatefulSet + Gateway + Cert + Route | ✅ Patterns defined | ❌ Not built |
| PostgreSQL StatefulSet (Bitnami Helm) | ✅ Spec complete | ❌ Not built |
| Ouranos web client Deployment + Gateway + Cert + Route | ✅ Patterns defined | ❌ Not built |
| Secret templates (envsubst) | ✅ Pattern from conduit | ❌ Not built |
| Relay / FeedGen manifests | Deferred to Phase 2 | ❌ Not needed yet |

### spec 03 (Agent Integration) — Deferred, no gaps for Phase 0

---

## Constraint Violations (Pre-existing)

| Constraint | Status | Action |
|------------|--------|--------|
| C-004: rsky-pds Dockerfile uses `ghcr.io/blacksky-algorithms` | ⚠️ WARN | Update Dockerfile to build from local context |
| C-007: Dockerfiles clone upstream repo at build time | ⚠️ WARN | Change all Dockerfiles to `COPY . .` pattern |

---

## Blocking Issues for Phase 0

1. **rsky-pds Dockerfile clones upstream** — must build from local context to pick up our env vars and config. The current Dockerfile does `RUN git clone https://github.com/blacksky-algorithms/rsky.git .` which means any local changes are ignored.

2. **No `k8s/` directory** — all manifests must be created from scratch. This is the primary deliverable of Phase 0.

3. **No deploy workflow** — `.github/workflows/deploy.yaml` must be created following the conduit pattern but adapted for ArgoCD (build+push images, commit SHA tag back to repo, let ArgoCD sync).

4. **GitHub org/repo name** — the `project.json` uses `ghcr.io/know-me-tools` as the registry. Confirm: is the GitHub org `know-me-tools`? The conduit workflow uses `ghcr.io/know-me-tools/conduit`, confirming this org exists.

5. **PostgreSQL deployment strategy** — rsky-pds requires PostgreSQL. Two options:
   - **Option A**: Bitnami PostgreSQL Helm chart (managed, HA-capable) — recommended
   - **Option B**: Plain StatefulSet with postgres image — simpler, matches conduit style
   
   Recommend **Option A** (Bitnami Helm) for production resilience.

6. **S3 / Blob storage** — rsky-pds requires S3-compatible storage for blobs. Need to decide:
   - **Option A**: Google Cloud Storage with HMAC keys (native to GKE)
   - **Option B**: MinIO deployed in-cluster (self-contained)
   - **Option C**: AWS S3 bucket (external, simple)
   
   Recommend **Option A** (GCS with HMAC) for GKE-native simplicity.

---

## Phase 0 Deliverables (Ordered)

### D-001: Fix Dockerfiles for local build context
Update `rsky-pds/Dockerfile` (and others) to build from `COPY . .` instead of cloning upstream.

### D-002: Create `k8s/` directory structure
```
k8s/
├── namespace.yaml
├── argocd/
│   └── application.yaml
├── postgresql/
│   └── values.yaml              (Bitnami Helm values)
├── rsky-pds/
│   ├── statefulset.yaml
│   ├── service.yaml
│   ├── pvc.yaml
│   ├── configmap.yaml
│   ├── secret.yaml              (envsubst template)
│   ├── gateway.yaml
│   ├── certificate.yaml
│   ├── httproute-https.yaml
│   └── httproute-redirect.yaml
├── web-client/                  (Ouranos)
│   ├── deployment.yaml
│   ├── service.yaml
│   ├── configmap.yaml
│   ├── gateway.yaml
│   ├── certificate.yaml
│   ├── httproute-https.yaml
│   └── httproute-redirect.yaml
└── ...
```

### D-003: Create GitHub Actions deploy workflow
`.github/workflows/deploy.yaml` — adapts conduit pattern for:
- Per-service Docker builds (matrix strategy)
- Image push to `ghcr.io/know-me-tools/<service>:<sha>`
- Commit image tag back to `k8s/<service>/*.yaml`
- ArgoCD auto-syncs (no `kubectl apply` in CI)

### D-004: Update openspec/specs/01-web-client.md
Change recommendation from social-app to Ouranos, document env vars.

### D-005: Commit all scaffolding to main
Ensure `.kbd-orchestrator/`, `openspec/`, `CLAUDE.md`, `k8s/`, deploy workflow are all in git.

---

## Open Questions Requiring Human Input

| # | Question | Impact |
|---|----------|--------|
| Q1 | Confirm GitHub org is `know-me-tools` for GHCR registry | Affects all Docker image tags and pull secrets |
| Q2 | GCS vs MinIO vs AWS S3 for blob storage? | Affects rsky-pds secret config |
| Q3 | Is the GKE service account already configured for GHCR pull access? | Affects `ghcr-pull-secret` setup in workflow |
| Q4 | Should PostgreSQL be Bitnami Helm chart or plain StatefulSet? | Affects Phase 0 complexity |
| Q5 | Domain DNS: who manages `know-me.tools` DNS? GCP Cloud DNS or external? | Affects how we document post-deploy DNS steps |

---

## Recommended Next Step

```
/kbd-plan phase-0-foundation
```

The plan will produce an ordered OpenSpec change list for the 5 deliverables above, assign each to the appropriate execution tool, and write the waypoint for Codex/Cursor/Antigravity to pick up.
