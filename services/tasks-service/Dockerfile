FROM rust:latest AS builder

WORKDIR /app

COPY . .

RUN cargo build --release --package tasks-service

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y libpq5 && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/tasks-service /usr/local/bin/

EXPOSE 3004

CMD ["tasks-service"]
