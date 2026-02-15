# --- Stage 1: Builder ---
FROM rust:latest as builder
WORKDIR /app
COPY . .
RUN cargo build --release

# --- Stage 2: Runtime ---
FROM rust:latest
WORKDIR /app

# Install GnuCOBOL, OpenSSL, and CA Certificates
RUN apt-get update && \
    apt-get install -y \
    ca-certificates \
    libssl-dev \
    gnucobol \
    libcob4-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy the compiled green_agent binary
COPY --from=builder /app/target/release/green_agent .

# Ensure the binary has execution permissions
RUN chmod +x ./green_agent

# Verify installations
RUN rustc --version && cobc --version

CMD ["./green_agent"]