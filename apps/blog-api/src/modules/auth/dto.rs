use app_foundation::i18n::{translate, MessageKey};
use app_foundation::{AppError, ErrorCode, Locale, ValidationDetail};
use serde::{Deserialize, Serialize};

use crate::modules::user::model::UserResponse;

/// 注册请求。
#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub name: String,
    pub email: String,
    pub password: String,
}

impl RegisterRequest {
    pub fn validate(&self, locale: Locale) -> Result<(), AppError> {
        if self.name.trim().is_empty() {
            return Err(AppError::BadRequestWithDetails(
                ErrorCode::UserNameEmpty,
                translate(locale, MessageKey::NameCannotBeEmpty).to_string(),
                vec![ValidationDetail::new("name", "required")],
            ));
        }

        if self.email.trim().is_empty() {
            return Err(AppError::BadRequestWithDetails(
                ErrorCode::UserEmailEmpty,
                translate(locale, MessageKey::EmailCannotBeEmpty).to_string(),
                vec![ValidationDetail::new("email", "required")],
            ));
        }

        if self.password.len() < 6 {
            return Err(AppError::BadRequestWithDetails(
                ErrorCode::UserPasswordTooShort,
                translate(locale, MessageKey::PasswordTooShort).to_string(),
                vec![ValidationDetail::new("password", "min_length_6")],
            ));
        }

        Ok(())
    }
}

/// 登录请求。
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

impl LoginRequest {
    pub fn validate(&self, locale: Locale) -> Result<(), AppError> {
        if self.email.trim().is_empty() {
            return Err(AppError::BadRequestWithDetails(
                ErrorCode::UserEmailEmpty,
                translate(locale, MessageKey::EmailCannotBeEmpty).to_string(),
                vec![ValidationDetail::new("email", "required")],
            ));
        }

        if self.password.is_empty() {
            return Err(AppError::BadRequestWithDetails(
                ErrorCode::UserPasswordEmpty,
                translate(locale, MessageKey::PasswordCannotBeEmpty).to_string(),
                vec![ValidationDetail::new("password", "required")],
            ));
        }

        Ok(())
    }
}

/// Refresh 请求。
#[derive(Debug, Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

impl RefreshRequest {
    pub fn validate(&self, locale: Locale) -> Result<(), AppError> {
        if self.refresh_token.trim().is_empty() {
            return Err(AppError::BadRequestWithDetails(
                ErrorCode::AuthMissingRefreshToken,
                translate(locale, MessageKey::MissingRefreshToken).to_string(),
                vec![ValidationDetail::new("refresh_token", "required")],
            ));
        }

        Ok(())
    }
}

/// Logout 请求。
#[derive(Debug, Deserialize)]
pub struct LogoutRequest {
    pub refresh_token: String,
}

impl LogoutRequest {
    pub fn validate(&self, locale: Locale) -> Result<(), AppError> {
        if self.refresh_token.trim().is_empty() {
            return Err(AppError::BadRequestWithDetails(
                ErrorCode::AuthMissingRefreshToken,
                translate(locale, MessageKey::MissingRefreshToken).to_string(),
                vec![ValidationDetail::new("refresh_token", "required")],
            ));
        }

        Ok(())
    }
}

/// Token 响应。
#[derive(Debug, Serialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub refresh_expires_in: i64,
}

/// 登录响应。
#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: TokenResponse,
    pub user: UserResponse,
}
