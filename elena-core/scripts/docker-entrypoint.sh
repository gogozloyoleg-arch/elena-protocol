#!/bin/sh
# Создаёт кошелёк, если его ещё нет, и запускает узел.
# ENV: NODE_NAME (alice|bob|charlie), LISTEN_PORT, ADMIN_PORT, PEERS (multiaddr через запятую), BALANCE

set -e
DATA_DIR="${DATA_DIR:-/data}"
NODE_NAME="${NODE_NAME:-alice}"
LISTEN_PORT="${LISTEN_PORT:-9000}"
ADMIN_PORT="${ADMIN_PORT:-9190}"
BALANCE="${BALANCE:-1000}"
PEERS="${PEERS:-}"

WALLET_PATH="${DATA_DIR}/wallets/${NODE_NAME}.key"
mkdir -p "${DATA_DIR}/wallets"

if [ ! -f "$WALLET_PATH" ]; then
  echo "Creating wallet: $NODE_NAME"
  elena-core -d "$DATA_DIR" wallet "$NODE_NAME"
fi

PEER_ARGS=""
if [ -n "$PEERS" ]; then
  for p in $(echo "$PEERS" | tr ',' ' '); do
    PEER_ARGS="$PEER_ARGS --peers $p"
  done
fi

# Глобальные -d, --listen, --peers должны идти до подкоманды run
exec elena-core -d "$DATA_DIR" --listen "/ip4/0.0.0.0/tcp/${LISTEN_PORT}" $PEER_ARGS run \
  --wallet "$NODE_NAME" \
  --balance "$BALANCE" \
  --admin "0.0.0.0:${ADMIN_PORT}"
