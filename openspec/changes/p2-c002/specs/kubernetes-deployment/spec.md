# Spec Delta: p2-c002 — kubernetes-deployment

## ADDED Requirements

### Requirement: Phase-0 and phase-1 work MUST land on `main` as discrete logical commits

The accumulated phase-0 and phase-1 work (Dockerfiles, k8s manifests, CI workflow, README updates) MUST be committed to `main` as a small set of logically grouped commits with conventional-commit messages, not a single mega-commit and not a long messy series.

#### Scenario: Commit history review

- **WHEN** an operator runs `git log --oneline main` after the phase-2 commit step
- **THEN** the phase-0 + phase-1 work is visible as a small number of commits each with a clear conventional-commit subject (e.g. `feat: add k8s manifests for rsky-pds`, `ci: add deploy workflow`), and no commit contains both unrelated services and infrastructure changes
