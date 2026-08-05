use sqlx::SqlitePool;

/// Возвращает текущего лидера недели или `None`, если лидер ещё не определён.
pub async fn get_leader(pool: &SqlitePool) -> Result<Option<String>, sqlx::Error> {
    sqlx::query_scalar("SELECT currentleader FROM currentleader LIMIT 1")
        .fetch_optional(pool)
        .await
}
