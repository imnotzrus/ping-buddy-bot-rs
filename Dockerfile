FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /app
RUN apt-get update && apt-get install -y --no-install-recommends \
    lld \
    clang \
    && rm -rf /var/lib/apt/lists/*

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .
RUN cargo build --release --bin ping-buddy

# We do not need the Rust toolchain to run the binary!
FROM debian:bookworm-slim AS runtime
WORKDIR /app
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends \
        openssl \
        ca-certificates \
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/ping-buddy /usr/local/bin
ENTRYPOINT ["/usr/local/bin/ping-buddy"]
