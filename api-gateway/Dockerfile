FROM rust:latest as builder

WORKDIR /usr/src/app

# Copy workspace Cargo files
COPY Cargo.toml Cargo.lock ./

# Copy shared library
COPY shared ./shared

# Copy api-gateway
COPY api-gateway ./api-gateway

# Create dummy main.rs files for workspace members to satisfy cargo
RUN mkdir -p services/user-service/src && \
    echo "fn main() {}" > services/user-service/src/main.rs && \
    mkdir -p services/project-service/src && \
    echo "fn main() {}" > services/project-service/src/main.rs && \
    mkdir -p services/billing-service/src && \
    echo "fn main() {}" > services/billing-service/src/main.rs && \
    mkdir -p services/notification-service/src && \
    echo "fn main() {}" > services/notification-service/src/main.rs && \
    mkdir -p services/taks-service/src && \
    echo "fn main() {}" > services/taks-service/src/main.rs && \
    mkdir -p frontend/src && \
    echo "fn main() {}" > frontend/src/main.rs

# Copy service Cargo.toml files
COPY services/user-service/Cargo.toml ./services/user-service/
COPY services/project-service/Cargo.toml ./services/project-service/
COPY services/billing-service/Cargo.toml ./services/billing-service/
COPY services/notification-service/Cargo.toml ./services/notification-service/
COPY services/taks-service/Cargo.toml ./services/taks-service/
COPY frontend/Cargo.toml ./frontend/

# Build the api-gateway
RUN cargo build --release --package api-gateway

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/app/target/release/api-gateway /usr/local/bin/api-gateway

ENV RUST_LOG=info

EXPOSE 3000

CMD ["api-gateway"]
