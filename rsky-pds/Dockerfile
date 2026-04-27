FROM rust:1.86-slim AS builder

RUN apt-get update && apt-get install -y \
    pkg-config libssl-dev libpq-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/rsky

# Copy workspace manifest and lock file for dependency caching
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./

# Stub all workspace member Cargo.toml files so Cargo resolves the full graph
COPY palomar-sync/Cargo.toml palomar-sync/
COPY rsky-common/Cargo.toml rsky-common/
COPY rsky-crypto/Cargo.toml rsky-crypto/
COPY rsky-feedgen/Cargo.toml rsky-feedgen/
COPY rsky-firehose/Cargo.toml rsky-firehose/
COPY rsky-identity/Cargo.toml rsky-identity/
COPY rsky-jetstream-subscriber/Cargo.toml rsky-jetstream-subscriber/
COPY rsky-labeler/Cargo.toml rsky-labeler/
COPY rsky-lexicon/Cargo.toml rsky-lexicon/
COPY rsky-pds/Cargo.toml rsky-pds/
COPY rsky-relay/Cargo.toml rsky-relay/
COPY rsky-repo/Cargo.toml rsky-repo/
COPY rsky-satnav/Cargo.toml rsky-satnav/
COPY rsky-syntax/Cargo.toml rsky-syntax/
COPY rsky-video/Cargo.toml rsky-video/
COPY rsky-wintermute/Cargo.toml rsky-wintermute/

# Create dummy src/main.rs (or lib.rs) for every member to satisfy Cargo
RUN for crate in palomar-sync rsky-common rsky-crypto rsky-feedgen rsky-firehose \
    rsky-identity rsky-jetstream-subscriber rsky-labeler rsky-lexicon rsky-pds \
    rsky-relay rsky-repo rsky-satnav rsky-syntax rsky-video rsky-wintermute; do \
      mkdir -p $crate/src && echo "fn main() {}" > $crate/src/main.rs; \
    done

# Cache all dependencies
RUN cargo build --release -p rsky-pds 2>/dev/null; true

# Now copy the real source and rebuild only what changed
COPY . .
RUN touch rsky-pds/src/main.rs && cargo build --release -p rsky-pds

# ── Runtime stage ─────────────────────────────────────────────────────────────
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates libssl3 libpq5 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /usr/src/rsky/target/release/rsky-pds .

LABEL org.opencontainers.image.source=https://github.com/know-me-tools/rsky

EXPOSE 3000
CMD ["./rsky-pds"]
