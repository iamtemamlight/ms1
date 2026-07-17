# ALLBRIGHT backend - Render.com production build
# Multi-stage Docker build for Rust backend
# Optimized for Render's Docker runtime

# ============================================================================
# Stage 1: Builder
# ============================================================================
FROM rust:1.96-slim AS builder

# Install system dependencies required for compilation
RUN apt-get update && apt-get install -y --no-install-recommends \
        build-essential \
        pkg-config \
        libssl-dev \
        libpq-dev \
        cmake \
        perl \
        make \
        ca-certificates \
        git \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# ============================================================================
# Cache dependencies - copy manifests first
# ============================================================================
COPY backend/Cargo.toml backend/Cargo.lock ./
COPY backend/build.rs ./
COPY backend/c2_service.proto ./c2_service.proto
COPY backend/signer.proto ./signer.proto

# Create dummy source tree for dependency caching
RUN mkdir -p src ai learning benches && \
    echo "fn main() {}" > src/main.rs && \
    echo "fn main() {}" > db_init.rs

# Build dependencies (cached layer)
RUN cargo build --release --bin allbright-c2-backend || true
RUN cargo build --release --bin db-init || true

# ============================================================================
# Copy source code and build
# ============================================================================
COPY backend/src ./src
COPY backend/ai ./ai
COPY backend/learning ./learning
COPY backend/benches ./benches
COPY backend/main.rs ./main.rs
COPY backend/db_init.rs ./db_init.rs

# Production build optimizations
ENV CARGO_PROFILE_RELEASE_LTO=fat
ENV CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1
ENV CARGO_PROFILE_RELEASE_PANIC=abort
ENV SQLX_OFFLINE=true

# Build the actual binaries
RUN cargo build --release --bin allbright-c2-backend
RUN cargo build --release --bin db-init

# ============================================================================
# Stage 2: Runtime (minimal image)
# ============================================================================
FROM debian:bookworm-slim

# Install only runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
        ca-certificates \
        libssl3 \
        libpq5 \
        curl \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 10001 -s /bin/bash appuser

# Set working directory
WORKDIR /app

# Copy binaries from builder stage
COPY --from=builder /app/target/release/allbright-c2-backend .
COPY --from=builder /app/target/release/db-init .

# Set ownership
RUN chown -R appuser:appuser /app

# Switch to non-root user
USER appuser

# Expose ports (Render uses 3001 for HTTP, health check at /healthz)
EXPOSE 3001 50051 50052

# Add health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3001/healthz || exit 1

# Run the backend
CMD ["./allbright-c2-backend"]
