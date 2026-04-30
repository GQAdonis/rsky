# Spec Delta: p1-c006 — kubernetes-deployment

## ADDED Requirements

### Requirement: CI workflow MUST build only services whose source changed

Per-service GitHub Actions workflows MUST use path filters so that, on a push that only touches one service's source, only that service's image is rebuilt and pushed. The repo's top-level README MUST document this CI shape.

#### Scenario: Single-service push triggers single-service build

- **WHEN** a commit pushed to `main` only modifies files under `rsky-relay/`
- **THEN** the rsky-relay build workflow runs and the rsky-pds, rsky-feedgen, rsky-labeler, rsky-jetstream-subscriber, and web-client build workflows do not run

### Requirement: Top-level README MUST document the deploy + CI shape for operators

The repository's top-level `README.md` MUST contain an "Operations" section that explains: which services exist, where their k8s manifests live, how CI builds and deploys them, and how to bootstrap the stack from scratch.

#### Scenario: New operator onboarding

- **WHEN** an operator unfamiliar with the stack reads the top-level README
- **THEN** they can identify the five rsky services, locate their k8s manifests, find the CI deploy workflow, and follow a documented bootstrap sequence
