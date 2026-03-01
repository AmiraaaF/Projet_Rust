FROM rust:latest AS builder

WORKDIR /app

COPY . .

RUN cargo build --release --package project-service

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y libpq5 && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/project-service /usr/local/bin/

EXPOSE 3002

CMD ["project-service"]
