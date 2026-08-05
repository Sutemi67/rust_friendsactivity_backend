use axum::Router;
use axum::routing::post;
use sqlx::SqlitePool;

pub mod handler;
pub mod model;

pub fn router() -> Router<SqlitePool> {
    Router::new()
        .route("/login", post(handler::login_user))
        .route("/login_update", post(handler::change_login))
}
