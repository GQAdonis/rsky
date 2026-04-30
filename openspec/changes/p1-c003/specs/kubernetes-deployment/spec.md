# Spec Delta: p1-c003 — kubernetes-deployment

## ADDED Requirements

### Requirement: rsky-relay MUST be deployed as a Deployment with its own Gateway at `relay.know-me.tools`

The rsky-relay workload MUST be deployed as a `Deployment` (stateless), exposed via a dedicated `Gateway` resource at `relay.know-me.tools`, with a cert-manager `Certificate` referencing `ClusterIssuer: letsencrypt`.

#### Scenario: Relay deploy produces all required manifests

- **WHEN** the k8s manifests for rsky-relay are applied to the `atproto` namespace
- **THEN** the namespace contains a `Deployment/rsky-relay`, a `Service`, a `Gateway` for `relay.know-me.tools`, an `HTTPRoute`, and a `Certificate` referencing `ClusterIssuer: letsencrypt`
