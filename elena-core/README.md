# Елена Core (Rust)

Пост-квантовая децентрализованная платёжная сеть с механизмом «Эхо-локации».

## Быстрый старт

```bash
# Сборка
cargo build --release

# Создать кошелёк (сохраняется в .elena/wallets/<name>.key)
cargo run -- wallet my_wallet

# Запуск узла с кошельком и локальным RPC (по умолчанию 127.0.0.1:9190)
cargo run -- run --balance 1000 --wallet my_wallet --admin 127.0.0.1:9190

# В другом терминале: статистика узла
cargo run -- stats --admin 127.0.0.1:9190

# Публичный ключ узла (для отправки ему платежей)
cargo run -- pubkey --admin 127.0.0.1:9190

# Платёж (сумма и комиссия в микро-ELENA; комиссия вычитается автоматически)
cargo run -- send --to <pubkey_hex> --amount 100 --admin 127.0.0.1:9190

# Стейкинг репутации (доля 0 .. 0.5)
cargo run -- stake --amount 0.3 --admin 127.0.0.1:9190

# Осаждение: запуск с начислением награды за хранение раз в час
cargo run -- run --wallet alice --admin 127.0.0.1:9190 --emission-interval 3600
```

Граф сохраняется в `data_dir/graph.json`. Комиссия по транзакциям: база + 0.01% от суммы; 50% комиссии получают узлы-хранители. Осаждение и стейкинг — см. **economics** и docs/TOKENOMICS.md.

## Архитектура

```
elena-core/
├── crypto/       # Dilithium3 + SHA3-512
├── economics/    # Tokenomics: комиссии, репутация, эмиссия (см. docs/TOKENOMICS.md)
├── graph/        # Транзакции, алерты, локальный граф
├── network/      # P2P на libp2p (TCP/QUIC, Noise, Floodsub)
├── consensus/    # Эхо-локация, детектор коллизий
└── node/         # Узел: баланс, репутация, обработка событий
```

## Криптография

- Подписи: **CRYSTALS-Dilithium3** (NIST)
- Хеши: **SHA3-512**

## Сеть (libp2p)

- **Транспорт:** TCP и QUIC, шифрование Noise, мультиплексирование Yamux
- **Pub/Sub:** Floodsub, топики `elena-transactions` и `elena-alerts`
- **Обнаружение:** опционально mDNS (фича `mdns`)

### Запуск полноценной сети (три узла)

Создайте кошельки и запустите узлы в **трёх терминалах**. Можно один раз выполнить `bash scripts/run_network.sh` — скрипт создаст кошельки и выведет команды для копирования.

```bash
# Узел 1 (Алиса)
cargo run -- wallet alice
cargo run -- run --wallet alice --admin 127.0.0.1:9190 --listen /ip4/0.0.0.0/tcp/9000

# Узел 2 (Боб)
cargo run -- wallet bob
cargo run -- run --wallet bob --admin 127.0.0.1:9191 --listen /ip4/0.0.0.0/tcp/9001 --peers /ip4/127.0.0.1/tcp/9000

# Узел 3 (Чарли)
cargo run -- wallet charlie
cargo run -- run --wallet charlie --admin 127.0.0.1:9192 --listen /ip4/0.0.0.0/tcp/9002 --peers /ip4/127.0.0.1/tcp/9000
```

После запуска в других терминалах:
- `cargo run -- stats --admin 127.0.0.1:9190` — статистика Алисы
- `cargo run -- stats --admin 127.0.0.1:9191` — Боба
- `cargo run -- send --to <pubkey_bob_hex> --amount 100 --admin 127.0.0.1:9190` — платёж от Алисы Бобу (pubkey Боба из вывода `cargo run -- wallet bob`)

Запуск двух узлов (без кошельков, с автогенерацией ключей):

```bash
# Терминал 1
cargo run -- run --listen /ip4/0.0.0.0/tcp/9000

# Терминал 2
cargo run -- run --listen /ip4/0.0.0.0/tcp/9001 --peers /ip4/127.0.0.1/tcp/9000
```

## Docker

Сеть из трёх узлов (Алиса, Боб, Чарли) одной командой:

```bash
cd elena-core
docker compose up -d
```

- Кошельки создаются автоматически в томах при первом запуске.
- RPC с хоста: `cargo run -- stats --admin 127.0.0.1:9190` (Алиса), `127.0.0.1:9191` (Боб), `127.0.0.1:9192` (Чарли).
- Остановка: `docker compose down`. Данные кошельков сохраняются в томах.

Публичный ключ узла для платежей: `cargo run -- pubkey --admin 127.0.0.1:9191` (Боб) и подставить в `send --to`.

## Тесты

```bash
cargo test
cargo test --test integration
```

## Зависимости

- Rust 1.70+ (через [rustup](https://rustup.rs)).
- **macOS:** нужны Xcode Command Line Tools и принятая лицензия. Если сборка падает с `You have not agreed to the Xcode license agreements`, выполните в терминале:
  ```bash
  sudo xcodebuild -license
  ```
  Прочитайте лицензию (пробел — вперёд, в конце введите `agree`). После этого снова запустите `cargo build`.

Если `cargo` не в PATH:
```bash
source $HOME/.cargo/env
# или
export PATH="$HOME/.cargo/bin:$PATH"
```
