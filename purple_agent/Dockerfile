# Stage 1: Build
FROM rust:1.85-bookworm as builder
WORKDIR /app
COPY . .
RUN cargo build --release

# Stage 2: Runtime
FROM debian:bookworm-slim
WORKDIR /app

# Purple Agent only needs CA certificates for Gemini API calls
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/purple_agent /app/purple_agent

EXPOSE 8081
CMD ["./purple_agent"]