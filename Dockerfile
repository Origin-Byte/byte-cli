# Stage 1: Installing the cargo-chef utility
FROM rust:1.70-slim-bullseye AS chef

# sui specific
# Install basic dependencies
# + cargo chef
RUN apt-get update \
    && DEBIAN_FRONTEND=noninteractive TZ=Etc/UTC apt-get install -y --no-install-recommends \
    tzdata \
    git-all \
    ca-certificates \
    curl \
    build-essential \
    libssl-dev \
    pkg-config \
    libclang-dev \
    cmake \
    && rm -rf /var/lib/apt/lists/* \
    && cargo install cargo-chef

WORKDIR /workspace/crates/bytes-api

# Stage 2: Preparing a list of dependencies using cargo-chef
FROM chef AS planner
WORKDIR /workspace
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
WORKDIR /workspace/crates/bytes-api
COPY --from=planner recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .
RUN cargo build --release
RUN cargo install --path . --verbose

# Create the final minimal image
FROM debian:bullseye-slim

# Copy the binary from the builder to the final image
COPY --from=builder /workspace/crates/bytes-api/target/release/byte-api .

# Specify the command to run your application
ENTRYPOINT ["/byte-api"]

