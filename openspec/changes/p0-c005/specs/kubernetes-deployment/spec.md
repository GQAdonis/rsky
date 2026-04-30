# Spec Delta: p0-c005 — kubernetes-deployment

## ADDED Requirements

### Requirement: GitHub Actions deploy workflow MUST build, push, and roll out per service

A GitHub Actions workflow at `.github/workflows/deploy.yaml` (or per-service equivalents) MUST, on push to `main`: build each affected service's image, push to `ghcr.io/know-me-tools/<service>` with both `${{ github.sha }}` and `latest` tags, render templated manifests via `envsubst`, apply them, and wait for `kubectl rollout status`.

#### Scenario: Push to main triggers full deploy

- **WHEN** a commit is pushed to `main` that touches `rsky-pds/src/`
- **THEN** the workflow builds the rsky-pds image, pushes both tags to `ghcr.io/know-me-tools/rsky-pds`, applies the manifests with secrets rendered via `envsubst`, and reports success only after `kubectl rollout status statefulset/rsky-pds` completes

### Requirement: ArgoCD Application MUST sync the cluster state from this repo's manifests

An ArgoCD `Application` resource MUST be configured to watch this repository's `k8s/` directory and reconcile cluster state with `automated.prune: true` and `automated.selfHeal: true`.

#### Scenario: Manifest deletion reflects in cluster

- **WHEN** a manifest file is removed from `k8s/` and pushed to `main`
- **THEN** ArgoCD prunes the corresponding live resource within its sync interval
