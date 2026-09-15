use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct RegisterInput {
    pub username: String,
    pub password: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub nickname: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LoginInput {
    pub account: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct UserDto {
    pub id: i32,
    pub username: String,
    pub nickname: String,
    pub avatar_url: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub role: String,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub user: UserDto,
}
