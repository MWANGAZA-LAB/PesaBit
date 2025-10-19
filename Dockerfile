# Base Rust image for building
FROM rust:1.75-slim as base

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Install cargo-watch for development
RUN cargo install cargo-watch

# Set working directory
WORKDIR /app

# Copy workspace configuration
COPY Cargo.toml Cargo.lock ./
COPY shared ./shared

# Development stage
FROM base as development

# Copy source code (will be overridden by volume mounts in development)
COPY services ./services

# Build dependencies (this layer will be cached)
RUN cargo build --workspace

# Expose port (will be overridden by service-specific Dockerfile)
EXPOSE 8000

# Default command (will be overridden by docker-compose)
CMD ["cargo", "run"]

# Production build stage
FROM base as builder

# Copy source code
COPY services ./services

# Build the application in release mode
RUN cargo build --release --workspace

# Runtime stage
FROM debian:bookworm-slim as runtime

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libpq5 \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN groupadd -r app && useradd -r -g app app

# Set working directory
WORKDIR /app

# Copy the built binary from builder stage
# (specific binary will be copied in service-specific Dockerfile)
COPY --from=builder /app/target/release/ ./bin/

# Change ownership to app user
RUN chown -R app:app /app
USER app

# Expose port
EXPOSE 8000

# Default command (will be overridden by service-specific Dockerfile)
CMD ["./bin/api-gateway"]