# Spec Delta: p2-c005 — kubernetes-deployment

## ADDED Requirements

### Requirement: First push to `main` MUST trigger a green CI run that produces all images and rolls out all services

The first push to `main` after secret seeding and ArgoCD bootstrap MUST cause the deploy workflow to: build every service image, push them to `ghcr.io/know-me-tools/*`, apply manifests, and complete `kubectl rollout status` for every workload, with the workflow run finishing green.

#### Scenario: Initial deploy is fully green

- **WHEN** the first push to `main` triggers the deploy workflow
- **THEN** every per-service build job succeeds, every image is pushed under both `<sha>` and `latest` tags, every `kubectl rollout status` command completes within its timeout, and the overall workflow run is green
