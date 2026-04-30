# Spec Delta: p1-c001 — service-images

## ADDED Requirements

### Requirement: rsky-relay MUST bind to `0.0.0.0` in container deployments

The `rsky-relay` binary MUST bind its listener to `0.0.0.0` (or the address provided via env var) when running in a Kubernetes pod, not `127.0.0.1`. Loopback-only binding makes the relay unreachable from peer pods and the Service.

#### Scenario: Relay reachable from in-cluster client

- **WHEN** a peer pod in the `atproto` namespace runs `curl http://rsky-relay:<port>/xrpc/_health`
- **THEN** the request succeeds (relay accepted the connection on a non-loopback address)

### Requirement: rsky-relay MUST ship a Dockerfile compatible with this workspace's image-build rules

The `rsky-relay/Dockerfile` MUST follow the same pattern as the other rsky service Dockerfiles: build from local workspace context, use `cargo build --release -p rsky-relay`, and target a slim runtime image.

#### Scenario: Relay image build

- **WHEN** GitHub Actions builds the rsky-relay image from this workspace
- **THEN** the build runs `cargo build --release -p rsky-relay` against local source and produces an image at `ghcr.io/know-me-tools/rsky-relay`
