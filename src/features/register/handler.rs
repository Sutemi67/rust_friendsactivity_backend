use axum::Json;
use axum::extract::State;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::error::ApiError;
use crate::repository::{tokens, users};

use super::model::{RegisterRequest, RegisterResponse};

pub async fn register_user(
    State(pool): State<SqlitePool>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<RegisterResponse>, ApiError> {
    let existing = users::get_user(&pool, &req.login).await?;
    if existing.is_some() {
        tracing::info!("Existing user try: {}", req.login);
        return Err(ApiError::UserAlreadyExists);
    }

    let token = Uuid::new_v4().to_string();

    if let Err(e) = users::insert(&pool, &req.login, &req.password).await {
        tracing::warn!("register insert failed: {e}");
        return Err(ApiError::RegisterFailed);
    }
    if let Err(e) = tokens::upsert(&pool, &req.login, &token).await {
        tracing::warn!("register token upsert failed: {e}");
        return Err(ApiError::RegisterFailed);
    }

    tracing::info!("Successful register happened: {}", req.login);
    Ok(Json(RegisterResponse { token }))
}
