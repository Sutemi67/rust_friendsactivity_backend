use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub login: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginChangeRequest {
    pub login: String,
    #[serde(rename = "newLogin")]
    pub new_login: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
}

#[derive(Debug, Serialize)]
pub struct LoginChangeResponse {
    pub message: String,
}
