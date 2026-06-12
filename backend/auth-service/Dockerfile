FROM rust:1.81-slim-bookworm AS builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY shared shared/
COPY auth-service auth-service/

RUN apt-get update && apt-get install -y pkg-config libssl-dev && \
    cargo build --release --bin auth-service && \
    cp target/release/auth-service /auth-service

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*
COPY --from=builder /auth-service /usr/local/bin/auth-service
EXPOSE 3001
CMD ["auth-service"]
