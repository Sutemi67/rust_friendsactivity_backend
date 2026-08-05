use sqlx::SqlitePool;

/// Сохраняет токен для логина, обновляя существующий при повторной записи.
pub async fn upsert(pool: &SqlitePool, login: &str, token: &str) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO tokens (login, token) VALUES (?, ?)
         ON CONFLICT(login) DO UPDATE SET token = excluded.token",
    )
    .bind(login)
    .bind(token)
    .execute(pool)
    .await?;
    Ok(())
}
