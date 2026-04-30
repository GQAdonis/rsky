# Spec Delta: p0-c001 — service-images

## ADDED Requirements

### Requirement: Service Dockerfiles MUST build from local workspace context

Every service Dockerfile in the rsky workspace MUST use `COPY . .` (or scoped equivalents) against the local checkout, not `RUN git clone https://github.com/blacksky-algorithms/rsky` or any other remote-clone pattern. CI must inject local changes into the build.

#### Scenario: rsky-pds image build uses local source

- **WHEN** GitHub Actions runs `docker build -f rsky-pds/Dockerfile .` in the cloned repository
- **THEN** the resulting image contains the workspace's current `rsky-pds/src/` content, not whatever is on `main` of `blacksky-algorithms/rsky`

### Requirement: Cargo workspace builds inside Docker MUST use per-crate `-p` filters

Image builds MUST NOT run `cargo build --workspace`. Each service Dockerfile MUST select only the crate it ships, e.g. `cargo build --release -p rsky-pds`, to avoid building all 17 workspace members for every image.

#### Scenario: Per-crate build filter

- **WHEN** any service Dockerfile compiles its binary
- **THEN** the `cargo build` command includes `-p <crate-name>` and the build does not compile unrelated workspace members
