# ELENA Wallet — Desktop (один клик для своего узла)

**Один EXE для Windows:** как собрать один установочный файл и скинуть другу — см. [ONE_EXE_WINDOWS.md](../docs/ONE_EXE_WINDOWS.md) в корне docs.

Приложение позволяет развернуть **свой узел и кошелёк** в один клик: при первом запуске создаётся кошелёк, запускаются узел и gateway, после чего можно открыть веб-кошелёк в браузере.

## Подготовка бинарников (для сборки приложения)

Узел и gateway поставляются вместе с приложением. Нужно собрать их и положить в `src-tauri/resources/bin/`:

**macOS / Linux (из корня репозитория):**
```bash
./scripts/build-node-bins.sh
```

**Вручную:**
```bash
cd elena-core && cargo build --release && cp target/release/elena-core ../elena-desktop/src-tauri/resources/bin/
cd elena-gateway && cargo build --release && cp target/release/elena-gateway ../elena-desktop/src-tauri/resources/bin/
```

**Windows:** соберите на Windows и скопируйте `elena-core.exe` и `elena-gateway.exe` в `elena-desktop/src-tauri/resources/bin/`.

## Запуск в режиме разработки

```bash
npm install
npm run tauri:dev
```

- При первом запуске соберите бинарники (см. выше), иначе кнопка «Запустить узел» покажет ошибку.
- Нажмите **«Запустить узел»** — создастся кошелёк (если его ещё нет), запустятся узел и gateway.
- Нажмите **«Открыть кошелёк»** — откроется веб-кошелёк в браузере по адресу http://localhost:9180.

## Сборка приложения

```bash
# Сначала положите бинарники в resources/bin/ (см. выше)
npm run tauri build
```

**Артефакты:**
- **macOS:** `src-tauri/target/release/bundle/macos/ELENA Wallet.app`, `.dmg`
- **Windows:** `src-tauri/target/release/bundle/msi/`, `nsis/` — установщики и .exe
- **Linux:** `.deb`, `AppImage` и др. в `src-tauri/target/release/bundle/`

Пользователь устанавливает приложение, запускает его, нажимает «Запустить узел» — после этого у него свой узел и свой кошелёк.

## Если приложение не открывается на macOS

1. **Неподписанное приложение:** правый клик по «ELENA Wallet» → **Открыть** → подтвердить.
2. **Режим разработки:** первый запуск `npm run tauri:dev` может занять 1–3 минуты (сборка Rust).

## Доступ к localhost

В бандл добавлен `Info.plist` с `NSAllowsLocalNetworking = true` для доступа к http://localhost:9180.
