# Capability: service-images

This capability defines how rsky service Docker images are built, tagged, and published.

## Purpose

The rsky workspace contains five services with Dockerfiles: `rsky-pds`, `rsky-relay`, `rsky-feedgen`, `rsky-labeler`, `rsky-jetstream-subscriber`, plus the Ouranos web client. Image builds must be deterministic, derived from local workspace state (not upstream `git clone`), and published to `ghcr.io/know-me-tools` with both immutable SHA tags and rolling `latest` tags.

## Requirements

### Requirement: Service Dockerfiles MUST build from local workspace context

Every service Dockerfile in the rsky workspace MUST use `COPY . .` (or scoped equivalents) against the local checkout, not `RUN git clone https://github.com/blacksky-algorithms/rsky` or any other remote-clone pattern. CI must inject local changes into the build.

#### Scenario: rsky-pds image build uses local source

- **WHEN** GitHub Actions runs `docker build -f rsky-pds/Dockerfile .` in the cloned repository
- **THEN** the resulting image contains the workspace's current `rsky-pds/src/` content, not whatever is on `main` of `blacksky-algorithms/rsky`

### Requirement: Cargo workspace builds inside Docker MUST use per-crate `-p` filters

Image builds MUST NOT run `cargo build --workspace`. Each service Dockerfile MUST select only the crate it ships, e.g. `cargo build --release -p rsky-pds`, to avoid building all 17 workspace members for every image.

#### Scenario: rsky-relay image excludes unrelated crates

- **WHEN** the rsky-relay Docker image is built
- **THEN** the build runs `cargo build --release -p rsky-relay` (and any required dependency crates) and does not compile `rsky-pds`, `rsky-feedgen`, or other unrelated workspace members

### Requirement: Images MUST be published with both immutable SHA tags and a rolling `latest` tag

Each successful main-branch build MUST push the image under two tags: `${{ github.sha }}` (immutable, used by k8s manifests) and `latest` (rolling, used for ad-hoc pulls).

#### Scenario: PDS image published with both tags

- **WHEN** the PDS build workflow completes successfully on `main`
- **THEN** `ghcr.io/know-me-tools/rsky-pds:<sha>` and `ghcr.io/know-me-tools/rsky-pds:latest` both exist and resolve to the same image digest

### Requirement: Image registry MUST be `ghcr.io/know-me-tools`

Service Dockerfiles, k8s manifests, and CI workflows MUST reference `ghcr.io/know-me-tools` as the image registry — never `ghcr.io/blacksky-algorithms` (the upstream registry).

#### Scenario: k8s manifest image reference

- **WHEN** a k8s `Deployment` or `StatefulSet` manifest references the rsky-pds image
- **THEN** the `image:` field starts with `ghcr.io/know-me-tools/rsky-pds:` and never with `ghcr.io/blacksky-algorithms/`
