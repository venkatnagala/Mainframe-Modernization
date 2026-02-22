FROM rust:1.91-slim-bookworm AS builder

WORKDIR /app

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release && rm -rf src

COPY src ./src
RUN touch src/main.rs && cargo build --release

# Runtime stage - keep full Rust toolchain for compiling user code
FROM rust:1.88-slim-bookworm

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl-dev \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/rust-mcp .

# Pre-warm cargo registry cache
RUN mkdir -p /tmp/cargo_home

RUN useradd -r -s /bin/false mcpuser
USER mcpuser

EXPOSE 8084

HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD wget -qO- http://localhost:8084/health || exit 1

CMD ["./rust-mcp"]
