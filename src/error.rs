use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

/// Типизированные ошибки API. Сообщения и статусы повторяют поведение Ktor-сервера 1:1.
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("User does not exists")]
    UserNotFound,
    #[error("Password is incorrect")]
    WrongPassword,
    #[error("User is already exists")]
    UserAlreadyExists,
    #[error("error due to nick changing")]
    LoginChangeFailed,
    #[error("Error in register action :(")]
    RegisterFailed,
    #[error("User does not exists, so cant fetch data")]
    UserNotFoundActivity,
    #[error(transparent)]
    Db(#[from] sqlx::Error),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = match &self {
            ApiError::UserAlreadyExists => StatusCode::CONFLICT,
            ApiError::UserNotFound
            | ApiError::WrongPassword
            | ApiError::LoginChangeFailed
            | ApiError::RegisterFailed
            | ApiError::UserNotFoundActivity => StatusCode::BAD_REQUEST,
            ApiError::Db(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, self.to_string()).into_response()
    }
}
