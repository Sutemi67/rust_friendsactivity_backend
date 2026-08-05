# AGENTS.md

## Project state

Rust-крят (lib + bin), edition 2024. Сервер-миграция с Ktor: HTTP API активности друзей.
Стек: `axum` + `tokio`, БД — `sqlx` (SQLite), сериализация — `serde`/`serde_json`.

- **Порт**: `6655` (переопределяется через `BIND_ADDR`).
- **API** (совместимость 1:1 с исходным Ktor-сервером, JSON — camelCase, ошибки — text/plain):
  - `POST /register` — `{login, password}` → `{token}` (200); 409 «User is already exists»
  - `POST /login` — `{login, password}` → `{token}` (200); 400 «User does not exists» / «Password is incorrect»
  - `POST /login_update` — `{login, newLogin}` → `{message: "ok!"}`; 400 «error due to nick changing»
  - `POST /post_activity` — `{login, steps, weeklySteps}` → `{friendsList, errorMessage, leader}` (200); 400 «User does not exists, so cant fetch data»
  - `GET /fetch/{login}` — `{steps, weeklySteps, errorMessage}` (всегда 200)
- **Конфигурация** (env): `BIND_ADDR` (по умолчанию `0.0.0.0:6655`), `DATABASE_URL`
  (по умолчанию `sqlite://friends_activity.db`), логирование через `RUST_LOG`.
- **БД**: SQLite, миграции — `migrations/` (встраиваются в бинарник через `sqlx::migrate!`).

## Structure

```
src/
├── main.rs          # точка входа: env → pool → миграции → планировщик → axum::serve
├── lib.rs           # экспорт модулей (для интеграционных тестов)
├── config.rs        # Config из env
├── db.rs            # SqlitePool + миграции
├── error.rs         # ApiError (thiserror) + IntoResponse
├── router.rs        # сборка всех роутов
├── features/        # feature-модули (как в Ktor)
│   ├── login/       # POST /login, /login_update
│   ├── register/    # POST /register
│   └── activity/    # POST /post_activity, GET /fetch/{login}, weekly_reset
└── repository/      # sqlx-запросы: users, tokens, leader
migrations/          # 0001_init.sql: users, tokens, currentleader
tests/api.rs         # интеграционные тесты API (tower oneshot + in-memory SQLite)
.github/workflows/   # build.yml: CI-сборка под Linux при пуше в main
```

## Commands

- `cargo build` / `cargo run` — сборка/запуск.
- `cargo test` — интеграционные тесты (`tests/api.rs`).
- `cargo fmt` / `cargo fmt --check` — форматирование.
- `cargo clippy --all-targets -- -D warnings` — линтер (должен быть чистым).
- `cargo build --release` — оптимизированный бинарник (~5 MB).

## Gotchas

- Edition 2024 требует Rust 1.85+; установленный toolchain — 1.93.
- `Cargo.lock` закоммичен; игнорируется только `/target`.
- `sqlx` использует фичи: `sqlite`, `runtime-tokio`, `migrate`, `derive`, `macros`
  (макрос `migrate!` требует `macros`; «голый» `use sqlx::migrate` — это модуль, не макрос).
- **Совместимость API критична** — не менять формат ответов/статусы/тексты ошибок
  без явного запроса. `Option`-поля должны сериализоваться в `null` (без `skip_serializing_if`),
  порядок полей в JSON сохранять.
- Особенности поведения, унаследованные от Ktor:
  - при `/login` токен генерируется, но **не** сохраняется в БД;
  - `/login_update` переименовывает сам `login` пользователя;
  - `/fetch` всегда отвечает 200 JSON, в т.ч. с `errorMessage`.
- Пароли хранятся в открытом виде (как в исходном Ktor-сервере) — осознанное решение.
- Часовой пояс сервера влияет на время еженедельного сброса (`weekly_reset`,
  понедельник 00:00, использует `chrono::Local`).
