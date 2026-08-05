use axum::Json;
use axum::extract::{Path, State};
use sqlx::SqlitePool;

use crate::error::ApiError;
use crate::repository::{leader, users};

use super::model::{UserActivity, UserDataResponse, UsersActivityResponse};

pub async fn update_steps(
    State(pool): State<SqlitePool>,
    Json(req): Json<UserActivity>,
) -> Result<Json<UsersActivityResponse>, ApiError> {
    let existing = users::get_user(&pool, &req.login).await?;

    if existing.is_none() {
        tracing::info!("User {} does not exists, so cant fetch data", req.login);
        return Err(ApiError::UserNotFoundActivity);
    }

    let friends_list = users::update_steps_and_list(&pool, &req).await?;
    let leader = leader::get_leader(&pool).await?;

    tracing::info!("User {} fetched data, steps: {}", req.login, req.steps);
    Ok(Json(UsersActivityResponse {
        friends_list,
        error_message: None,
        leader,
    }))
}

pub async fn get_user_data(
    State(pool): State<SqlitePool>,
    Path(login): Path<String>,
) -> Json<UserDataResponse> {
    match users::get_user(&pool, &login).await {
        Ok(Some(user)) => Json(UserDataResponse {
            steps: Some(user.steps),
            weekly_steps: Some(user.weekly_steps),
            error_message: None,
        }),
        Ok(None) => Json(UserDataResponse {
            steps: None,
            weekly_steps: None,
            error_message: Some("User not found".to_string()),
        }),
        Err(e) => {
            tracing::warn!("Connecting error for {}: {e}", login);
            Json(UserDataResponse {
                steps: None,
                weekly_steps: None,
                error_message: Some(format!("Connecting error. {e}")),
            })
        }
    }
}
