# Статус деплоя

## ✅ Сделано

### 1. GitHub
- **Репозиторий:** https://github.com/gogozloyoleg-arch/elena-protocol
- Код загружен, ветка `main`

### 2. Веб-кошелёк (Vercel)
- **URL:** https://elena-web-three.vercel.app
- Страница «Создать узел», Настройки с URL API, ссылки на `gogozloyoleg-arch/elena-protocol`

---

## 📋 Что нужно сделать вручную

### 1. Развернуть узел + gateway на Render (2 минуты)

1. Открой: **https://render.com/deploy?repo=https://github.com/gogozloyoleg-arch/elena-protocol**
2. Войди в Render (через GitHub).
3. Нажми **Deploy** (или **Apply**).
4. Дождись сборки (~5–10 минут).
5. Скопируй URL сервиса (например, `https://elena-node-gateway-xxx.onrender.com`).

### 2. Подключить gateway к кошельку

**Вариант A — для всех пользователей**

1. Vercel → проект elena-web → Settings → Environment Variables.
2. Добавь переменную: `VITE_API_URL` = URL твоего Render (например, `https://elena-node-gateway-xxx.onrender.com`).
3. Redeploy проекта.

**Вариант B — для конкретного пользователя**

1. Открыть https://elena-web-three.vercel.app.
2. Настройки → URL API → вставить URL Render.
3. Сохранить.

### 3. (Опционально) Автодеплой Vercel при push

1. Vercel → проект elena-web → Settings → Git.
2. Подключи репозиторий `gogozloyoleg-arch/elena-protocol`.
3. Root Directory: `elena-web`.
4. После этого каждый push в main будет автоматически деплоить.

---

## Ссылки

| Сервис        | URL |
|---------------|-----|
| Кошелёк       | https://elena-web-three.vercel.app |
| Создать узел  | https://elena-web-three.vercel.app/create-node |
| GitHub        | https://github.com/gogozloyoleg-arch/elena-protocol |
| Render Deploy | https://render.com/deploy?repo=https://github.com/gogozloyoleg-arch/elena-protocol |
