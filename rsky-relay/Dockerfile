FROM lukemathwalker/cargo-chef:latest-rust-1.86 AS chef
RUN apt-get update && apt-get install -y \
    pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /usr/src/rsky

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /usr/src/rsky/recipe.json recipe.json
RUN cargo chef cook --release -p rsky-relay --recipe-path recipe.json
COPY . .
RUN cargo build --release -p rsky-relay

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
    ca-certificates libssl3 procps \
    && rm -rf /var/lib/apt/lists/*
LABEL org.opencontainers.image.source=https://github.com/know-me-tools/rsky
# WORKDIR is the data directory so relative paths (relay.db, plc_directory.db)
# resolve to the PVC-backed mount point.
WORKDIR /data
COPY --from=builder /usr/src/rsky/target/release/rsky-relay /usr/local/bin/rsky-relay
EXPOSE 9000
CMD ["rsky-relay"]
