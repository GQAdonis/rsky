FROM rust:1.86-slim AS chef
RUN cargo install cargo-chef --locked
RUN apt-get update && apt-get install -y \
    pkg-config libssl-dev libpq-dev \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /usr/src/rsky

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /usr/src/rsky/recipe.json recipe.json
RUN cargo chef cook --release -p rsky-labeler --recipe-path recipe.json
COPY . .
RUN cargo build --release -p rsky-labeler

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
    ca-certificates libssl3 libpq5 \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /usr/src/rsky/target/release/rsky-labeler .
LABEL org.opencontainers.image.source=https://github.com/know-me-tools/rsky
CMD ["./rsky-labeler"]
