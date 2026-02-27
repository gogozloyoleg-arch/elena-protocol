# Деплой ELENA Web Wallet

## Вариант A: Vercel CLI (с вашего компьютера)

1. **Войти в Vercel** (один раз):
   ```bash
   cd elena-web
   npx vercel login
   ```
   Откроется браузер для входа через GitHub/GitLab/Email.

2. **Деплой (preview — тестовый URL):**
   ```bash
   npx vercel
   ```
   Ответьте на вопросы: Link to existing project? **N** → Project name: **elena-web** (или свой) → Directory: **./** (Enter).

3. **Деплой в production:**
   ```bash
   npx vercel --prod
   ```
   Или после первого шага: `npm run deploy`.

4. **Переменная для API:**
   - В [vercel.com/dashboard](https://vercel.com/dashboard) → ваш проект → **Settings** → **Environment Variables**
   - Добавьте: **VITE_API_URL** = `https://ваш-gateway.example.com` (без слеша в конце)
   - Передеплойте: **Deployments** → ⋮ у последнего → **Redeploy**.

---

## Вариант B: GitHub + Vercel (деплой при каждом push)

1. **Создайте репозиторий на GitHub** (если ещё нет):
   ```bash
   cd "/Users/macbook/Протокол Елена"
   git add .
   git commit -m "ELENA: web wallet, gateway, docker"
   git remote add origin https://github.com/ВАШ_ЛОГИН/elena.git
   git branch -M main
   git push -u origin main
   ```

2. **Подключите репозиторий к Vercel:**
   - [vercel.com/new](https://vercel.com/new) → **Import Git Repository** → выберите репозиторий.
   - **Root Directory:** нажмите **Edit** → укажите **elena-web** → **Continue**.
   - **Environment Variables:** добавьте **VITE_API_URL** (URL вашего API Gateway) → **Deploy**.

3. Дальше при каждом `git push` в ветку по умолчанию Vercel будет собирать и деплоить фронт автоматически.

---

## После деплоя

- Ссылка на сайт будет в формате: `https://elena-web-xxx.vercel.app` или свой домен в настройках.
- Кошелёк будет ходить на API по адресу из **VITE_API_URL**. Убедитесь, что API Gateway доступен по HTTPS и в нём настроен CORS для домена Vercel (в `elena-gateway` уже стоит `allow_origin(Any)`).
