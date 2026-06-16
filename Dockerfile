# syntax=docker/dockerfile:1

FROM rust:bookworm AS builder
WORKDIR /workspace
COPY Cargo.toml ./
RUN mkdir src && echo 'fn main() {}' > src/main.rs && echo 'pub fn lib() {}' > src/lib.rs
RUN cargo build --release 2>/dev/null || true
COPY src ./src
RUN cargo build --release

FROM debian:bookworm-slim AS runtime
ARG TARGETARCH
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates curl docker.io \
    && rm -rf /var/lib/apt/lists/* \
    && ARCH=$(case "$TARGETARCH" in amd64) echo x86_64 ;; arm64) echo aarch64 ;; *) echo "$TARGETARCH" ;; esac) \
    && mkdir -p /usr/local/lib/docker/cli-plugins \
    && curl -fsSL "https://github.com/docker/compose/releases/download/v2.29.0/docker-compose-linux-${ARCH}" \
        -o /usr/local/lib/docker/cli-plugins/docker-compose \
    && chmod +x /usr/local/lib/docker/cli-plugins/docker-compose
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
