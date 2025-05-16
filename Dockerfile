FROM rust:1.87.0-alpine AS base

# Install required dependencies including static SSL libraries
RUN apk add --no-cache \
    musl-dev \
    pkgconfig \
    openssl-dev \
    openssl-libs-static \
    build-base \
    perl \
    cmake

# Use cargo-binstall to install precompiled binaries and avoid compilation issues
RUN wget -qO- https://github.com/cargo-bins/cargo-binstall/releases/latest/download/cargo-binstall-x86_64-unknown-linux-musl.tgz | tar -xzf - -C /usr/local/cargo/bin

# Install tools using binstall when possible, fallback to normal install
RUN cargo binstall --no-confirm sccache || \
    RUSTFLAGS="-C target-feature=+crt-static" cargo install sccache --version ^0.7 --no-default-features --features=gcs,s3,redis

RUN cargo binstall --no-confirm cargo-chef || \
    cargo install cargo-chef --version ^0.1

ENV RUSTC_WRAPPER=sccache SCCACHE_DIR=/sccache

# Planner stage - create recipe.json with dependency information
FROM base AS planner
WORKDIR /app
COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry \
  --mount=type=cache,target=$SCCACHE_DIR,sharing=locked \
  cargo chef prepare --recipe-path recipe.json

# Builder stage - build dependencies separately for caching
FROM base AS builder
WORKDIR /app
COPY --from=planner /app/recipe.json recipe.json

# Cook dependencies
RUN --mount=type=cache,target=/usr/local/cargo/registry \
  --mount=type=cache,target=$SCCACHE_DIR,sharing=locked \
  cargo chef cook --release --target x86_64-unknown-linux-musl --recipe-path recipe.json

# Copy source and build the application with musl target
COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry \
  --mount=type=cache,target=$SCCACHE_DIR,sharing=locked \
  cargo build --release --target x86_64-unknown-linux-musl

# Truly minimal image with no dependencies
FROM scratch
WORKDIR /app
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/nmcp /app/
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/

# Set the entrypoint to the statically linked binary
ENTRYPOINT ["/app/nmcp"]
CMD ["--help"]
