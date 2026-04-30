# Spec Delta: p2-c004 — kubernetes-deployment

## ADDED Requirements

### Requirement: ArgoCD Application bootstrap MUST be applied once before first deploy

Before the first push to `main` that triggers a deploy, the operator MUST apply the ArgoCD `Application` manifest that watches this repository's `k8s/` directory. The manifest MUST be tracked in this repo and reference the repo's HTTPS URL.

#### Scenario: Bootstrap application present

- **WHEN** the operator runs `kubectl apply -f k8s/argocd/application.yaml` once
- **THEN** ArgoCD registers an `Application` watching this repo's `k8s/` path on `main` with `automated.prune: true` and `automated.selfHeal: true`, and from that point forward cluster state is reconciled with git automatically
