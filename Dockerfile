# Build stage
FROM rust:1.82-slim AS builder

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./
COPY crates/jiaodai-core/Cargo.toml crates/jiaodai-core/Cargo.toml
COPY crates/jiaodai-seal/Cargo.toml crates/jiaodai-seal/Cargo.toml
COPY crates/jiaodai-unseal/Cargo.toml crates/jiaodai-unseal/Cargo.toml
COPY crates/jiaodai-match/Cargo.toml crates/jiaodai-match/Cargo.toml
COPY crates/jiaodai-chain/Cargo.toml crates/jiaodai-chain/Cargo.toml
COPY crates/jiaodai-api/Cargo.toml crates/jiaodai-api/Cargo.toml
COPY crates/jiaodai-cli/Cargo.toml crates/jiaodai-cli/Cargo.toml

# Create dummy sources for dependency caching
RUN mkdir -p crates/jiaodai-core/src && echo "" > crates/jiaodai-core/src/lib.rs && \
    mkdir -p crates/jiaodai-seal/src && echo "" > crates/jiaodai-seal/src/lib.rs && \
    mkdir -p crates/jiaodai-unseal/src && echo "" > crates/jiaodai-unseal/src/lib.rs && \
    mkdir -p crates/jiaodai-match/src && echo "" > crates/jiaodai-match/src/lib.rs && \
    mkdir -p crates/jiaodai-chain/src && echo "" > crates/jiaodai-chain/src/lib.rs && \
    mkdir -p crates/jiaodai-api/src && echo "" > crates/jiaodai-api/src/lib.rs && \
    mkdir -p crates/jiaodai-cli/src && echo "fn main() {}" > crates/jiaodai-cli/src/main.rs

# Build dependencies only (cached layer)
RUN cargo build --release 2>/dev/null || true

# Copy actual source code
COPY . .

# Build the actual application
RUN cargo build --release -p jiaodai-cli

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/jiaodai /app/jiaodai

# Create data directory
RUN mkdir -p /app/data

ENV JIAODAI_PORT=3000
ENV JIAODAI_DB_PATH=/app/data/jiaodai.db
ENV JIAODAI_LOG_LEVEL=info

EXPOSE 3000

CMD ["/app/jiaodai"]
