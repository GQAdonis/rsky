FROM lukemathwalker/cargo-chef:latest-rust-1.86 AS chef
RUN apt-get update && apt-get install -y \
    pkg-config libssl-dev libpq-dev \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /usr/src/rsky

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /usr/src/rsky/recipe.json recipe.json
# Dependencies are cached until Cargo.toml / Cargo.lock changes
RUN cargo chef cook --release -p rsky-pds --recipe-path recipe.json
COPY . .
RUN cargo build --release -p rsky-pds

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
    ca-certificates libssl3 libpq5 \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /usr/src/rsky/target/release/rsky-pds .
LABEL org.opencontainers.image.source=https://github.com/know-me-tools/rsky
EXPOSE 3000
CMD ["./rsky-pds"]
