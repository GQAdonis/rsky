# p0-c005: GitHub Actions Deploy Workflow + ArgoCD Application

**Phase**: phase-0-foundation  
**Priority**: 5 (depends on all k8s manifests existing — p0-c001 through p0-c004)  
**Assigned to**: claude-code  

## Overview

Two deliverables in one change:
1. `.github/workflows/deploy.yaml` — builds images, commits SHA tags, triggers ArgoCD sync
2. `k8s/argocd/application.yaml` — ArgoCD Application pointing at `k8s/` in this repo

## ArgoCD GitOps Flow

```
git push to main
  → GitHub Actions: build Docker images (matrix per service)
  → GitHub Actions: push images to ghcr.io/know-me-tools/<service>:<sha>
  → GitHub Actions: sed-replace IMAGE_TAG in k8s manifests → commit back to main
  → ArgoCD: detects new commit → syncs atproto namespace automatically
```

This is different from conduit (which does `kubectl apply` directly). No `kubectl` in CI — ArgoCD owns cluster state.

## ArgoCD Application Manifest

```yaml
# k8s/argocd/application.yaml
apiVersion: argoproj.io/v1alpha1
kind: Application
metadata:
  name: atproto-stack
  namespace: argocd
spec:
  project: default
  source:
    repoURL: https://github.com/know-me-tools/rsky
    targetRevision: main
    path: k8s
    directory:
      recurse: true
      exclude: 'argocd/**'    # avoid ArgoCD managing itself
  destination:
    server: https://kubernetes.default.svc
    namespace: atproto
  syncPolicy:
    automated:
      prune: true
      selfHeal: true
    syncOptions:
      - CreateNamespace=true
      - ServerSideApply=true
```

## Deploy Workflow Structure

```yaml
# .github/workflows/deploy.yaml
name: Build & Deploy

on:
  push:
    branches: [main]
  workflow_dispatch:

env:
  GKE_CLUSTER: client-cluster
  GKE_REGION: us-central1
  REGISTRY: ghcr.io/know-me-tools

jobs:
  build-and-push:
    strategy:
      matrix:
        service: [rsky-pds, rsky-feedgen, rsky-labeler, rsky-jetstream-subscriber, web-client]
    # build each service image, push :sha and :latest

  update-image-tags:
    needs: build-and-push
    # sed replace IMAGE_TAG in each k8s/<service>/*.yaml
    # git commit "chore: update image tags [skip ci]"
    # git push
    # ArgoCD auto-detects and syncs

  verify-sync:
    needs: update-image-tags
    # authenticate to GKE
    # argocd app wait atproto-stack --timeout 300
    # OR: kubectl rollout status on key deployments

  print-dns-instructions:
    needs: verify-sync
    # echo Cloudflare DNS records to set
```

## Key Differences from Conduit Workflow

| Conduit | rsky (this) |
|---------|-------------|
| `kubectl apply -f k8s/...` in CI | ArgoCD syncs from git |
| Single service | Matrix build (5 services) |
| Image tag in statefulset sed | Image tag committed back to repo |
| No `argocd` CLI needed | `argocd app wait` for health check |
| GKE credentials in every deploy step | GKE credentials only for verify step |

## GHCR Pull Secret

Reuse same approach as conduit:
```yaml
- name: Create GHCR image pull secret
  run: |
    kubectl create secret docker-registry ghcr-pull-secret \
      --namespace atproto \
      --docker-server=ghcr.io \
      --docker-username=${{ github.actor }} \
      --docker-password=${{ secrets.GHCR_PAT }} \
      --dry-run=client -o yaml | kubectl apply -f -
```

## GitHub Secrets Required

| Secret | Value |
|--------|-------|
| `GKE_SA_KEY` | Same GCP service account JSON as conduit |
| `GKE_PROJECT_ID` | Same GCP project as conduit |
| `GHCR_PAT` | Same PAT as conduit |
| `POSTGRES_USER` | `rsky` |
| `POSTGRES_PASSWORD` | Generated strong password |
| `PDS_ADMIN_PASS` | Generated |
| `PDS_JWT_KEY_K256_PRIVATE_KEY_HEX` | Generated secp256k1 private key |
| `PDS_PLC_ROTATION_KEY_K256_PRIVATE_KEY_HEX` | Generated |
| `PDS_REPO_SIGNING_KEY_K256_PRIVATE_KEY_HEX` | Generated |
| `PDS_MAILGUN_API_KEY` | Mailgun API key |
| `PDS_MAILGUN_DOMAIN` | Mailgun domain |
| `GCS_HMAC_ACCESS_KEY` | GCS HMAC access key |
| `GCS_HMAC_SECRET_KEY` | GCS HMAC secret |
| `GCS_BUCKET_NAME` | GCS bucket for PDS blobs |
