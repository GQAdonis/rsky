# p0-c001: Fix Dockerfiles — Local Build Context

**Phase**: phase-0-foundation  
**Priority**: 1 (blocks all image builds)  
**Assigned to**: claude-code  

## Problem

All existing Dockerfiles (`rsky-pds`, `rsky-feedgen`, `rsky-labeler`, `rsky-jetstream-subscriber`, `rsky-firehose`) clone from `github.com/blacksky-algorithms/rsky` at build time. This means:
- Our local changes are ignored
- Images are built from upstream, not our fork
- CI cannot inject our environment-specific config

## Change

Replace the upstream `git clone` build pattern with a standard `COPY . .` multi-stage build for each Dockerfile.

## Pattern

```dockerfile
FROM rust:1.86 AS builder
WORKDIR /usr/src/rsky

# Copy workspace root files first for layer caching
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./

# Copy all workspace member manifests (dummy src trick for cache)
COPY rsky-pds/Cargo.toml rsky-pds/
RUN mkdir -p rsky-pds/src && echo "fn main() {}" > rsky-pds/src/main.rs
# ... repeat for each dependency crate ...

# Cache dependencies
RUN cargo build --release -p rsky-pds 2>/dev/null || true

# Copy real source and build
COPY . .
RUN touch rsky-pds/src/main.rs && cargo build --release -p rsky-pds

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /usr/src/rsky/target/release/rsky-pds .
EXPOSE 3000
CMD ["./rsky-pds"]
```

## Files to Change

- `rsky-pds/Dockerfile`
- `rsky-feedgen/Dockerfile`
- `rsky-labeler/Dockerfile`
- `rsky-jetstream-subscriber/Dockerfile`
- `rsky-firehose/Dockerfile`

## Verification

`docker build -f rsky-pds/Dockerfile .` from the workspace root succeeds.
