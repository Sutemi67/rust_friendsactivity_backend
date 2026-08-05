use axum::Json;
use axum::extract::State;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::error::ApiError;
use crate::repository::users;

use super::model::{LoginChangeRequest, LoginChangeResponse, LoginRequest, LoginResponse};

pub async fn login_user(
    State(pool): State<SqlitePool>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, ApiError> {
    let user = users::get_user(&pool, &req.login).await?;

    let Some(user) = user else {
        tracing::info!("User {} does not exists", req.login);
        return Err(ApiError::UserNotFound);
    };

    if user.password != req.password {
        tracing::info!("User {} password incorrect", req.login);
        return Err(ApiError::WrongPassword);
    }

    let token = Uuid::new_v4().to_string();
    tracing::info!("User {} - successful login, got a new token", req.login);
    Ok(Json(LoginResponse { token }))
}

pub async fn change_login(
    State(pool): State<SqlitePool>,
    Json(req): Json<LoginChangeRequest>,
) -> Result<Json<LoginChangeResponse>, ApiError> {
    let user = users::get_user(&pool, &req.login).await?;

    if user.is_none() {
        tracing::info!("User {} does not exists", req.login);
        return Err(ApiError::UserNotFound);
    }

    users::change_login(&pool, &req.login, &req.new_login)
        .await
        .map_err(|e| {
            tracing::warn!("User {} error in nick change: {e}", req.login);
            ApiError::LoginChangeFailed
        })?;

    tracing::info!("User {} changed login to {}", req.login, req.new_login);
    Ok(Json(LoginChangeResponse {
        message: "ok!".to_string(),
    }))
}
