# ELENA Web Wallet

Web-кошелёк для сети «Елена» (React + Vite).

## Локально

```bash
npm install
npm run dev
```

Откройте http://localhost:5173. Кошелёк обращается к API по адресу из `VITE_API_URL` или `http://localhost:9180`.

## Деплой на Vercel

Пошагово: **[DEPLOY.md](./DEPLOY.md)** (CLI и GitHub).

Кратко:
1. **Репозиторий:** залейте проект в GitHub (или подключите существующий в Vercel).

2. **Импорт в Vercel:**
   - [vercel.com](https://vercel.com) → **Add New** → **Project** → выберите репозиторий с `elena-web`.
   - **Root Directory:** оставьте `elena-web` или укажите папку, где лежит этот проект (если монорепо).
   - **Framework Preset:** Vite (подставится автоматически по `vercel.json`).

3. **Переменная окружения:**
   - **Environment Variables** → добавьте:
   - Имя: `VITE_API_URL`
   - Значение: URL вашего API Gateway, например `https://api.elena.example.com` или `https://your-gateway.railway.app`.
   - Важно: без завершающего слеша.

4. **Deploy:** нажмите **Deploy**. После сборки будет ссылка вида `https://elena-web-xxx.vercel.app`.

### Важно

- **Фронт** отдаётся с Vercel (статический хостинг).
- **API Gateway** должен быть развёрнут отдельно (ваш сервер, Railway, Fly.io, другой VPS). Браузер обращается к нему по `VITE_API_URL`; настройте CORS на gateway для домена Vercel.

## Обновить деплой на Vercel

После изменений в коде задеплойте заново:

```bash
cd elena-web
npx vercel --prod
```

Или: `npm run deploy`. Если проект подключён к GitHub — сделайте `git push`, Vercel соберёт и задеплоит сам.

## Сборка вручную

```bash
npm run build
```

Артефакты в `dist/`. Для продакшена задайте `VITE_API_URL` перед сборкой.
