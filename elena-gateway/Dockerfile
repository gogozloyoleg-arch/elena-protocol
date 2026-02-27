FROM rust:bookworm AS builder
WORKDIR /build
COPY Cargo.toml Cargo.lock* ./
COPY src ./src
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /build/target/release/elena-gateway /usr/local/bin/elena-gateway
ENV GATEWAY_LISTEN=0.0.0.0:9180
ENV ELENA_NODE_ADMIN=127.0.0.1:9190
EXPOSE 9180
CMD ["elena-gateway"]
