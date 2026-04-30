# Spec Delta: p1-c004 — kubernetes-deployment

## ADDED Requirements

### Requirement: rsky-feedgen MUST be deployed as a Deployment with its own Gateway at `feed.know-me.tools`

The rsky-feedgen workload MUST be deployed as a `Deployment` (stateless), exposed via a dedicated `Gateway` resource at `feed.know-me.tools`, with a cert-manager `Certificate` referencing `ClusterIssuer: letsencrypt`. It MUST connect to the in-cluster Postgres `rsky_feedgen` database.

#### Scenario: Feedgen deploy produces all required manifests

- **WHEN** the k8s manifests for rsky-feedgen are applied to the `atproto` namespace
- **THEN** the namespace contains a `Deployment/rsky-feedgen`, a `Service`, a `Gateway` for `feed.know-me.tools`, an `HTTPRoute`, and a `Certificate` referencing `ClusterIssuer: letsencrypt`, and the deployment's env vars point its database connection at the cluster's `postgres` Service against the `rsky_feedgen` database
