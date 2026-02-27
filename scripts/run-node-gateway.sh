#!/bin/sh
# Запуск узла и gateway в одном контейнере
set -e
export DATA_DIR="${DATA_DIR:-/data}"
export NODE_NAME="${NODE_NAME:-mynode}"
export LISTEN_PORT="${LISTEN_PORT:-9000}"
export ADMIN_PORT="${ADMIN_PORT:-9190}"
export BALANCE="${BALANCE:-1000}"

mkdir -p "$DATA_DIR/wallets"
WALLET_PATH="$DATA_DIR/wallets/${NODE_NAME}.key"
if [ ! -f "$WALLET_PATH" ]; then
  elena-core -d "$DATA_DIR" wallet "$NODE_NAME"
fi
elena-core -d "$DATA_DIR" --listen "/ip4/0.0.0.0/tcp/${LISTEN_PORT}" run \
  --wallet "$NODE_NAME" --balance "$BALANCE" --admin "0.0.0.0:${ADMIN_PORT}" &
sleep 2
exec elena-gateway
