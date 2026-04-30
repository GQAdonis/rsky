FROM rust:1.86 AS builder
RUN apt-get update && apt-get install -y \
    pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /usr/src/rsky

COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY rsky-syntax/ rsky-syntax/
COPY rsky-crypto/ rsky-crypto/
COPY rsky-identity/ rsky-identity/
COPY rsky-common/ rsky-common/
COPY rsky-lexicon/ rsky-lexicon/
COPY rsky-repo/ rsky-repo/
COPY rsky-firehose/ rsky-firehose/
COPY rsky-appview/ rsky-appview/

WORKDIR /usr/src/rsky/rsky-appview
RUN cargo build --release -p appview-bin

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
    ca-certificates libssl3 procps \
    && rm -rf /var/lib/apt/lists/*
LABEL org.opencontainers.image.source=https://github.com/know-me-tools/rsky
WORKDIR /data
COPY --from=builder /usr/src/rsky/rsky-appview/target/release/appview /usr/local/bin/appview
EXPOSE 8080
CMD ["appview"]
