use axum::Router;
use axum::routing::{get, post};
use sqlx::SqlitePool;

pub mod handler;
pub mod model;
pub mod weekly_reset;

pub fn router() -> Router<SqlitePool> {
    Router::new()
        .route("/post_activity", post(handler::update_steps))
        .route("/fetch/{login}", get(handler::get_user_data))
}
