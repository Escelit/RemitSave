FROM rust:1.81-slim-bookworm AS builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY shared shared/
COPY event-watcher event-watcher/

RUN apt-get update && apt-get install -y pkg-config libssl-dev && \
    cargo build --release --bin event-watcher && \
    cp target/release/event-watcher /event-watcher

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*
COPY --from=builder /event-watcher /usr/local/bin/event-watcher
CMD ["event-watcher"]
