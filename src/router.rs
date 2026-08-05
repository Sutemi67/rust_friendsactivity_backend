use axum::Router;
use sqlx::SqlitePool;

use crate::features;

/// Собирает роуты всех feature-модулей в единый роутер с общим состоянием.
pub fn build_router(pool: SqlitePool) -> Router {
    Router::new()
        .merge(features::register::router())
        .merge(features::login::router())
        .merge(features::activity::router())
        .with_state(pool)
}
