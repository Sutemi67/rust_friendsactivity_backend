use serde::{Deserialize, Serialize};

/// Данные активности пользователя: используется как тело запроса
/// `POST /post_activity` и как элемент `friendsList` в ответе.
#[derive(Debug, Deserialize, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct UserActivity {
    pub login: String,
    pub steps: i64,
    pub weekly_steps: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UsersActivityResponse {
    pub friends_list: Vec<UserActivity>,
    pub error_message: Option<String>,
    pub leader: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserDataResponse {
    pub steps: Option<i64>,
    pub weekly_steps: Option<i64>,
    pub error_message: Option<String>,
}
