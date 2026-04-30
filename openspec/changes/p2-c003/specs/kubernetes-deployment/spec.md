# Spec Delta: p2-c003 — kubernetes-deployment

## ADDED Requirements

### Requirement: All deploy-time secrets MUST be seeded as GitHub Actions secrets before first deploy

Every `${VAR}` placeholder referenced in any `secret.yaml` template MUST have a corresponding GitHub Actions secret configured before the first `main` push that triggers a deploy. Missing secrets MUST cause CI to fail at the `envsubst` step rather than apply an incomplete manifest.

#### Scenario: Missing secret fails the deploy

- **WHEN** the deploy workflow runs and a required secret is unset
- **THEN** the `envsubst` step (or a pre-apply check) detects the unsubstituted `${VAR}` placeholder and fails the workflow before `kubectl apply`

#### Scenario: Documented secret list

- **WHEN** an operator looks for which secrets must be configured
- **THEN** the repository's deploy documentation lists the full set of GitHub Actions secrets required (PDS keys, DB credentials, Mailgun, ghcr.io PAT, etc.)
