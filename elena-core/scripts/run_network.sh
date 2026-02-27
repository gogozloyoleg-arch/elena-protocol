#!/usr/bin/env bash
# Создаёт кошельки alice, bob, charlie и выводит команды для запуска трёх узлов.
# Запускать из корня elena-core: bash scripts/run_network.sh

set -e
cd "$(dirname "$0")/.."
DATA_DIR="${ELENA_DATA_DIR:-.elena}"

echo "=== Кошельки (data_dir=$DATA_DIR) ==="
cargo run -q -- wallet alice
cargo run -q -- wallet bob
cargo run -q -- wallet charlie

echo ""
echo "=== Запустите в трёх разных терминалах ==="
echo ""
echo "# Терминал 1 — Алиса"
echo "cargo run -- run --wallet alice --admin 127.0.0.1:9190 --listen /ip4/0.0.0.0/tcp/9000"
echo ""
echo "# Терминал 2 — Боб"
echo "cargo run -- run --wallet bob --admin 127.0.0.1:9191 --listen /ip4/0.0.0.0/tcp/9001 --peers /ip4/127.0.0.1/tcp/9000"
echo ""
echo "# Терминал 3 — Чарли"
echo "cargo run -- run --wallet charlie --admin 127.0.0.1:9192 --listen /ip4/0.0.0.0/tcp/9002 --peers /ip4/127.0.0.1/tcp/9000"
echo ""
echo "Статистика: cargo run -- stats --admin 127.0.0.1:9190  (или 9191, 9192)"
echo "Платёж:     cargo run -- send --to <pubkey_hex> --amount 100 --admin 127.0.0.1:9190"
