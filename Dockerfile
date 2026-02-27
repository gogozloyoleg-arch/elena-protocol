# Один образ: узел + gateway — для Render и деплоя
# Render ищет Dockerfile в корне

FROM rust:bookworm AS builder
WORKDIR /build

# Сборка elena-core
COPY elena-core/Cargo.toml elena-core/Cargo.lock* elena-core/
COPY elena-core/src elena-core/src
RUN cd elena-core && cargo build --release

# Сборка elena-gateway
COPY elena-gateway/Cargo.toml elena-gateway/Cargo.lock* elena-gateway/
COPY elena-gateway/src elena-gateway/src
RUN cd elena-gateway && cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /build/elena-core/target/release/elena-core /usr/local/bin/
COPY --from=builder /build/elena-gateway/target/release/elena-gateway /usr/local/bin/
COPY scripts/run-node-gateway.sh /run.sh
RUN chmod +x /run.sh

ENV DATA_DIR=/data
ENV NODE_NAME=mynode
ENV LISTEN_PORT=9000
ENV ADMIN_PORT=9190
ENV BALANCE=1000
ENV GATEWAY_LISTEN=0.0.0.0:9180
ENV ELENA_NODE_ADMIN=127.0.0.1:9190
ENV PORT=9180

VOLUME /data
EXPOSE 9180

CMD ["/run.sh"]
