# Сборка elena-core (Rust)
FROM rust:bookworm AS builder
WORKDIR /build

COPY Cargo.toml Cargo.lock* ./
COPY src ./src

RUN cargo build --release

# Минимальный образ для запуска
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /build/target/release/elena-core /usr/local/bin/elena-core
COPY scripts/docker-entrypoint.sh /docker-entrypoint.sh
RUN chmod +x /docker-entrypoint.sh

ENV DATA_DIR=/data
VOLUME /data

ENTRYPOINT ["/docker-entrypoint.sh"]
