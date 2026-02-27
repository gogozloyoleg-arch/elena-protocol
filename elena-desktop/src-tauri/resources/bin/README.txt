Скопируйте сюда бинарники elena-core и elena-gateway для вашей ОС:
- macOS/Linux: elena-core, elena-gateway
- Windows: elena-core.exe, elena-gateway.exe

Соберите их из корня проекта:
  cd elena-core && cargo build --release && cp target/release/elena-core ../elena-desktop/src-tauri/resources/bin/
  cd elena-gateway && cargo build --release && cp target/release/elena-gateway ../elena-desktop/src-tauri/resources/bin/

Или запустите скрипт: ./scripts/build-node-bins.sh
