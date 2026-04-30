# Spec Delta: p0-c004 — web-client

## ADDED Requirements

### Requirement: Ouranos web client MUST be deployed as a stateless Deployment with dedicated Gateway

The web client workload MUST be a `Deployment` (stateless, horizontally scalable), exposed via a dedicated `Gateway` resource at `social.know-me.tools`, with a cert-manager `Certificate` referencing `ClusterIssuer: letsencrypt`.

#### Scenario: Web client manifests applied

- **WHEN** the k8s manifests for the web client are applied to the `atproto` namespace
- **THEN** the namespace contains a `Deployment/web-client`, a `Service`, a `Gateway` for `social.know-me.tools`, an `HTTPRoute`, and a `Certificate` referencing `ClusterIssuer: letsencrypt`

### Requirement: Web client manifests MUST NOT bundle server-side PDS credentials

The web client `Deployment` and any associated `Secret` MUST contain only `NEXT_PUBLIC_*` env vars and other client-safe configuration. PDS admin passwords, JWT signing keys, and PLC rotation keys MUST NOT appear in any web client manifest.

#### Scenario: Manifest secret review

- **WHEN** all `Secret` resources referenced by the web client `Deployment` are inspected
- **THEN** none contain `PDS_ADMIN_PASS`, `PDS_JWT_KEY_*`, `PDS_REPO_SIGNING_KEY_*`, or `PDS_PLC_ROTATION_KEY_*`
