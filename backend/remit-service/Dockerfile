FROM rust:1.81-slim-bookworm AS builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY shared shared/
COPY remit-service remit-service/

RUN apt-get update && apt-get install -y pkg-config libssl-dev && \
    cargo build --release --bin remit-service && \
    cp target/release/remit-service /remit-service

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*
COPY --from=builder /remit-service /usr/local/bin/remit-service
EXPOSE 3002
CMD ["remit-service"]
