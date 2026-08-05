use sqlx::SqlitePool;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

/// Создаёт пул подключений к SQLite, при необходимости создавая файл БД.
pub async fn connect(database_url: &str) -> Result<SqlitePool, sqlx::Error> {
    let options = database_url
        .parse::<SqliteConnectOptions>()?
        .create_if_missing(true);

    SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await
}

/// Применяет миграции из папки `migrations`.
pub async fn run_migrations(pool: &SqlitePool) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!("./migrations").run(pool).await
}
