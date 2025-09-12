# Multi-stage build for ZephyrFS Node
FROM rust:1.81-slim as builder

# Install system dependencies for building
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy Cargo files first for better layer caching
COPY Cargo.toml Cargo.lock ./

# Create dummy main.rs to build dependencies
RUN mkdir -p src && echo "fn main() {}" > src/main.rs
RUN cargo build --release && rm -rf src

# Copy actual source code
COPY src/ src/

# Build the actual application
RUN cargo build --release

# Final runtime image
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN groupadd -r zephyr && useradd -r -g zephyr zephyr

# Create data directory
RUN mkdir -p /var/lib/zephyrfs && chown zephyr:zephyr /var/lib/zephyrfs

# Copy binary from builder stage
COPY --from=builder /app/target/release/zephyrfs-node /usr/local/bin/

USER zephyr

# Expose P2P and API ports
EXPOSE 4001 8080

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD zephyrfs-node --health-check || exit 1

ENTRYPOINT ["zephyrfs-node"]