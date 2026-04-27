# p2-c005: Push to main — first CI run

**Phase**: phase-2-commit-and-deploy  
**Priority**: 5 (depends on p2-c002, p2-c003, p2-c004)  
**Assigned to**: human  
**Type**: operational runbook

## Overview

Push the committed work to `main`. This triggers the GitHub Actions workflow:
`build-and-push` → `update-image-tags` → `inject-secrets` → `verify-sync`

Expected duration: ~10-15 minutes end-to-end.

## Command

```bash
git push origin main
```

## Monitoring

1. GitHub → Actions tab → watch "Build & Deploy" workflow
2. `build-and-push`: 6 parallel matrix jobs (~6-8 min each, cached after first run)
3. `update-image-tags`: commits SHA tags back to `k8s/**` manifests (~30s)
4. `inject-secrets`: applies all secrets to cluster (~1 min)
5. `verify-sync`: waits for ArgoCD Healthy + rollout status (~3 min)

## Common Failure Points

| Step | Failure | Fix |
|------|---------|-----|
| build-and-push | GHCR auth failure | Check `GHCR_PAT` has `packages:write` |
| build-and-push | Cargo build failure | Check Rust toolchain / Dockerfile |
| update-image-tags | Push auth failure | Check `GHCR_PAT` has `contents:write` |
| inject-secrets | kubectl auth failure | Check `GKE_SA_KEY` / `GKE_PROJECT_ID` |
| inject-secrets | envsubst missing var | Check all 17 secrets are seeded |
| verify-sync | ArgoCD timeout | Check ArgoCD Application exists (p2-c004) |
| verify-sync | rollout timeout | Check pod logs for crash loops |

## Post-Push Verification

```bash
# Check images were pushed
gh api /orgs/know-me-tools/packages?package_type=container | jq '.[].name'

# Check tags were committed back
git pull origin main && git log --oneline -3

# Check pods starting
kubectl get pods -n atproto -w
```
