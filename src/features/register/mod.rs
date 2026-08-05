use axum::Router;
use axum::routing::post;
use sqlx::SqlitePool;

pub mod handler;
pub mod model;

pub fn router() -> Router<SqlitePool> {
    Router::new().route("/register", post(handler::register_user))
}
