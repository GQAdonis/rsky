# Spec Delta: p0-c003 — kubernetes-deployment

## ADDED Requirements

### Requirement: rsky-pds MUST be deployed as a StatefulSet with dedicated Gateway and TLS

The rsky-pds workload MUST be a `StatefulSet` (stable storage identity for repo state), exposed via a dedicated `Gateway` resource at `pds.know-me.tools`, with a cert-manager `Certificate` referencing `ClusterIssuer: letsencrypt`.

#### Scenario: PDS deploy produces all required manifests

- **WHEN** the k8s manifests for rsky-pds are applied to the `atproto` namespace
- **THEN** the namespace contains a `StatefulSet/rsky-pds`, a `Service`, a `Gateway` resource scoped to `pds.know-me.tools`, an `HTTPRoute` binding the service, and a `Certificate` resource referencing `ClusterIssuer: letsencrypt`

### Requirement: rsky-pds MUST receive its config via env vars, not config files

The PDS Rocket configuration MUST come from environment variables (e.g., `ROCKET_PORT`, `ROCKET_ADDRESS`, plus all `PDS_*` env vars) so the container can be reconfigured without rebuilding the image.

#### Scenario: Container port override

- **WHEN** the StatefulSet sets `ROCKET_PORT=2583` in `containers[].env`
- **THEN** rsky-pds binds to port 2583 without any baked-in config file change
