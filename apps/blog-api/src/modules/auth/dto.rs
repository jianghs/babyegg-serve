use serde::{Deserialize, Serialize};

use crate::modules::user::model::UserResponse;

/// 注册请求。
#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub name: String,
    pub email: String,
    pub password: String,
}

/// 登录请求。
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

/// Token 响应。
#[derive(Debug, Serialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
}

/// 登录响应。
#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: TokenResponse,
    pub user: UserResponse,
}
