# syntax=docker/dockerfile:1

FROM rust:bookworm AS builder
WORKDIR /workspace
COPY Cargo.toml ./
RUN mkdir src && echo 'fn main() {}' > src/main.rs && echo 'pub fn lib() {}' > src/lib.rs
RUN cargo build --release 2>/dev/null || true
COPY src ./src
RUN cargo build --release

FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates docker.io docker-compose-plugin \
    && rm -rf /var/lib/apt/lists/*
RUN useradd -m -u 1000 mcp
COPY --from=builder /workspace/target/release/docker-mcp /usr/local/bin/docker-mcp
USER mcp
ENTRYPOINT ["docker-mcp"]

FROM rust:bookworm AS dev
WORKDIR /workspace
RUN rustup component add rustfmt clippy llvm-tools-preview
RUN cargo install cargo-llvm-cov
COPY Cargo.toml ./
RUN mkdir src && echo 'fn main() {}' > src/main.rs && echo 'pub fn lib() {}' > src/lib.rs
RUN cargo fetch
CMD ["bash"]
