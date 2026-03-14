# Build stage
FROM rust:latest AS builder

# Install build essentials
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy Cargo files first for dependency caching
COPY Cargo.toml ./

# Create a dummy Cargo.lock if it doesn't exist
# (This allows building even without lockfile)
RUN touch Cargo.lock

# Copy source
COPY src ./src

# Build release binary
RUN cargo build --release

# Final stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    wget \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -m -u 1000 appuser

# Create app directory
WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/chetna /app/chetna

# Create data directory
RUN mkdir -p /app/ChetnaData && chown -R appuser:appuser /app

# Switch to non-root user
USER appuser

# Expose port
EXPOSE 1987

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:1987/health || exit 1

# Environment variables with defaults
ENV CHETNA_PORT=1987
ENV CHETNA_DB_PATH=./ChetnaData/chetna.db
ENV EMBEDDING_BASE_URL=http://localhost:11434
ENV EMBEDDING_MODEL=qwen3-embedding:4b
ENV LLM_BASE_URL=http://localhost:11434
ENV LLM_MODEL=qwen3.5:4b
ENV CONSOLIDATION_INTERVAL=6

# Run the application
CMD ["/app/chetna"]
