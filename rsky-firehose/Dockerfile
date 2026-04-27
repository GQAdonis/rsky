FROM rust:1.86-slim AS builder

RUN apt-get update && apt-get install -y \
    pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/rsky

COPY Cargo.toml Cargo.lock rust-toolchain.toml ./

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

RUN for crate in palomar-sync rsky-common rsky-crypto rsky-feedgen rsky-firehose \
    rsky-identity rsky-jetstream-subscriber rsky-labeler rsky-lexicon rsky-pds \
    rsky-relay rsky-repo rsky-satnav rsky-syntax rsky-video rsky-wintermute; do \
      mkdir -p $crate/src && echo "fn main() {}" > $crate/src/main.rs; \
    done

RUN cargo build --release -p rsky-firehose 2>/dev/null; true

COPY . .
RUN touch rsky-firehose/src/main.rs && cargo build --release -p rsky-firehose

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /usr/src/rsky/target/release/rsky-firehose .

LABEL org.opencontainers.image.source=https://github.com/know-me-tools/rsky

CMD ["./rsky-firehose"]
