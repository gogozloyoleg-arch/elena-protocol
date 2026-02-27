#!/usr/bin/env bash
# Собирает elena-core и elena-gateway и копирует в elena-desktop/src-tauri/resources/bin/
# Запускайте из корня репозитория. Для Windows соберите на Windows и скопируйте .exe вручную.

set -e
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BIN_DIR="$ROOT/elena-desktop/src-tauri/resources/bin"
mkdir -p "$BIN_DIR"

echo "Building elena-core..."
(cd "$ROOT/elena-core" && cargo build --release)
echo "Building elena-gateway..."
(cd "$ROOT/elena-gateway" && cargo build --release)

if [[ "$OSTYPE" == "msys" || "$OSTYPE" == "win32" ]]; then
  cp "$ROOT/elena-core/target/release/elena-core.exe" "$BIN_DIR/"
  cp "$ROOT/elena-gateway/target/release/elena-gateway.exe" "$BIN_DIR/"
  echo "Copied elena-core.exe and elena-gateway.exe to $BIN_DIR"
else
  cp "$ROOT/elena-core/target/release/elena-core" "$BIN_DIR/"
  cp "$ROOT/elena-gateway/target/release/elena-gateway" "$BIN_DIR/"
  echo "Copied elena-core and elena-gateway to $BIN_DIR"
fi
