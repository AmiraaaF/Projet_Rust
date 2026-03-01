FROM rust:latest AS builder

WORKDIR /app

COPY . .

RUN cargo build --release --package user-service

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y libpq5 && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/user-service /usr/local/bin/

EXPOSE 3001

CMD ["user-service"]
