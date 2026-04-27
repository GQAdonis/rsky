# Plan: phase-2-commit-and-deploy

**Date**: 2026-04-27  
**Backend**: openspec  
**Phase goal**: Commit all phases 0+1 work, add Ouranos submodule, push to main, first successful CI run, ArgoCD bootstrap, DNS configuration, endpoint smoke test.

---

## Change Order

| ID | Title | Agent | Automatable | Depends on |
|----|-------|-------|-------------|------------|
| p2-c001 | Add Ouranos git submodule | claude-code | YES | — |
| p2-c002 | Commit all phases 0+1 work | claude-code | YES | p2-c001 |
| p2-c003 | Seed GitHub Actions secrets | human | NO (secrets) | p2-c002 |
| p2-c004 | Bootstrap ArgoCD Application | human | NO (kubectl access) | p2-c002 |
| p2-c005 | Push to main — first CI run | human | NO (CI trigger) | p2-c003, p2-c004 |
| p2-c006 | Configure Cloudflare DNS | human | NO (DNS admin) | p2-c005 |
| p2-c007 | Smoke test all endpoints | human | NO (live infra) | p2-c006 |

---

## Change Detail

### p2-c001 — Add Ouranos git submodule
- Run `git submodule add https://github.com/sudoWright/ouranos_atproto web-client`
- Creates `.gitmodules` at repo root
- Populates `web-client/` with Ouranos checkout
- Verify `Dockerfile.web-client` `COPY web-client/ .` will resolve correctly
- **Blocker**: CI `actions/checkout@v4 submodules: recursive` requires `.gitmodules` entry

### p2-c002 — Commit all phases 0+1 work
Three logical commits:

**Commit 1: `feat: add rsky-relay bind patch and Dockerfile`**
- `rsky-relay/src/server/server.rs` (bind `0.0.0.0`)
- `rsky-relay/Dockerfile`

**Commit 2: `feat: rewrite service Dockerfiles for k8s deployment`**
- `rsky-pds/Dockerfile`
- `rsky-feedgen/Dockerfile`
- `rsky-labeler/Dockerfile`
- `rsky-jetstream-subscriber/Dockerfile`
- `rsky-firehose/Dockerfile`
- `Dockerfile.web-client`

**Commit 3: `feat: add k8s GitOps stack for atproto namespace`**
- `.gitignore` (CLAUDE.md exclusion removed)
- `CLAUDE.md`
- `k8s/` (all manifests)
- `.github/workflows/deploy.yaml`
- `.github/scripts/update-image-tags.sh`
- `.kbd-orchestrator/` (all orchestration files)
- `openspec/` (all change management files)

### p2-c003 — Seed GitHub Actions secrets (human task)
Set all secrets under repository Settings → Secrets → Actions.
See `k8s/README.md` for complete list. Labeler secrets use placeholder values.

### p2-c004 — Bootstrap ArgoCD Application (human task)
```bash
kubectl apply -f k8s/argocd/application.yaml
```
One-time manual step. ArgoCD will not sync until this object exists.

### p2-c005 — Push to main — first CI run (human task)
```bash
git push origin main
```
Monitor GitHub Actions: build → update-image-tags → inject-secrets → verify-sync.
Expected duration: ~8-12 min. Watch for inject-secrets failures (missing secrets).

### p2-c006 — Configure Cloudflare DNS (human task)
After ArgoCD syncs and Envoy Gateways get external IPs:
```bash
kubectl get gateway -n atproto -o wide
```
Set Cloudflare A records (DNS-only / grey cloud):
- `pds.know-me.tools` → pds-gateway IP
- `relay.know-me.tools` → relay-gateway IP  
- `feed.know-me.tools` → feedgen-gateway IP
- `social.know-me.tools` → web-client-gateway IP

Wait for cert-manager to issue certificates (~2-3 min after DNS propagates).

### p2-c007 — Smoke test all endpoints (human task)
```bash
# PDS health
curl https://pds.know-me.tools/xrpc/_health

# Relay health
curl https://relay.know-me.tools/_health

# Feedgen health
curl https://feed.know-me.tools/xrpc/_health

# Web client
curl -I https://social.know-me.tools
```
All should return 2xx. PDS should return `{"version":"<semver>"}`.

---

## Constraints Checked

- C-001: No secrets committed — all via envsubst templates ✓
- C-003: All secrets have matching `secret.yaml` templates ✓
- C-004: All manifests reference `ghcr.io/know-me-tools` ✓
- C-007: All Gateways have matching Certificate resources ✓
- C-008: CI verify-sync job has rollout status waits ✓

---

## OpenSpec Changes to Emit

- `openspec/changes/p2-c001/` — submodule task
- `openspec/changes/p2-c002/` — commit task (3 logical commits)
- `openspec/changes/p2-c003/` — secrets runbook (human)
- `openspec/changes/p2-c004/` — ArgoCD bootstrap runbook (human)
- `openspec/changes/p2-c005/` — first push + CI runbook (human)
- `openspec/changes/p2-c006/` — DNS runbook (human)
- `openspec/changes/p2-c007/` — smoke test runbook (human)
