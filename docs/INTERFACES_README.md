# Интерфейсы сети «Елена»

Краткая инструкция по запуску всех компонентов.

## Быстрый старт (Docker)

Из корня репозитория:

```bash
docker compose up -d
```
(или `docker-compose up -d`, если установлен отдельный Compose V1)

После сборки:

- **Web-кошелёк:** http://localhost:3000  
- **Панель владельца:** http://localhost:3001  
- **API Gateway:** http://localhost:9180  

**Обновить образы после изменений:** `docker compose build web-wallet owner-dashboard` затем `docker compose up -d`.

Деплой Web-кошелька на Vercel: см. **elena-web/README.md**. Обновление: `cd elena-web && npx vercel --prod`. Создание кошельков и защита: **docs/WALLET_AND_SECURITY.md**.

Узлы: node1 (хосты 19000, 19190), node2 (19001, 19191). Web/API: 3000, 3001, 9180. Gateway подключён к node1.

## Локальная разработка

### 1. Узел (elena-core)

```bash
cd elena-core
cargo run -d /tmp/elena-data --listen /ip4/0.0.0.0/tcp/9000 run --wallet alice --admin 0.0.0.0:9190
```

### 2. API Gateway

```bash
cd elena-gateway
export ELENA_NODE_ADMIN=127.0.0.1:9190
cargo run
# Слушает 0.0.0.0:9180
```

### 3. Web-кошелёк

```bash
cd elena-web
npm install && npm run dev
# Откройте http://localhost:5173, в .env задайте VITE_API_URL=http://localhost:9180 при необходимости
```

### 4. Панель владельца

```bash
cd elena-owner-dashboard
npm install && npm run dev
# VITE_API_URL=http://localhost:9180
```

### 5. Desktop (Tauri, macOS)

```bash
cd elena-desktop
npm install
# Запустите узел и gateway, затем:
npm run tauri:dev
```

Если приложение не открывается: см. `elena-desktop/README.md` (Gatekeeper, первый запуск компиляции, запуск из терминала).

### 6. Мобильное приложение

```bash
cd elena-mobile
npm install
npm run start   # в одном терминале
npm run ios     # или npm run android — в другом
```

Для Android/iOS укажите в `src/services/api.ts` URL вашего gateway (например через ngrok при тесте с устройства).

---

## Пакетные менеджеры (формулы/репозитории)

Установка **elena-core** для пользователей:

### Homebrew (macOS)

После публикации репозитория:

```bash
brew tap elena/elena
brew install elena
elena -d ~/.elena wallet mywallet
elena -d ~/.elena --listen /ip4/0.0.0.0/tcp/9000 run --wallet mywallet --admin 0.0.0.0:9190
```

### APT (Ubuntu/Debian)

```bash
echo "deb https://repo.elena.network/apt stable main" | sudo tee /etc/apt/sources.list.d/elena.list
sudo apt update && sudo apt install elena
```

### Chocolatey (Windows)

```powershell
choco install elena
```

### NPM (CLI для веб-разработчиков)

Планируется пакет `elena-cli` для вызова RPC и работы с кошельком из терминала.

---

## Структура репозитория

| Каталог | Описание |
|--------|----------|
| `elena-core/` | Узел сети, CLI, кошелёк, RPC |
| `elena-gateway/` | REST + WebSocket API к узлу |
| `elena-web/` | Web-кошелёк (Vite + React) |
| `elena-owner-dashboard/` | Панель владельца сети |
| `elena-desktop/` | Desktop-клиент (Tauri + React) |
| `elena-mobile/` | Мобильное приложение (React Native) |
| `docker-compose.yml` | Узлы + gateway + web + dashboard |
