use app_foundation::i18n::{translate, MessageKey};
use app_foundation::{AppError, ErrorCode, Locale, ValidationDetail, ValidationReason};
use serde::{Deserialize, Serialize};

use crate::modules::identity::model::UserResponse;

/// 注册请求。
///
/// 用于 `POST /auth/register`。
/// 成功后会创建用户记录，并由身份域补充分配默认 `user` 角色。
#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    /// 注册用户名。
    pub name: String,
    /// 注册邮箱。
    pub email: String,
    /// 明文密码。
    pub password: String,
}

/// 登录请求。
///
/// 用于 `POST /auth/login`。
/// 该请求只校验必填性，邮箱存在性与密码正确性由身份域服务负责。
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    /// 登录邮箱。
    pub email: String,
    /// 明文密码。
    pub password: String,
}

impl LoginRequest {
    /// 校验登录请求的必填字段。
    ///
    /// 当前规则：
    /// - `email` 不能为空白字符串
    /// - `password` 不能为空字符串
    pub fn validate(&self, locale: Locale) -> Result<(), AppError> {
        if self.email.trim().is_empty() {
            return Err(AppError::BadRequestWithDetails(
                ErrorCode::UserEmailEmpty,
                translate(locale, MessageKey::EmailCannotBeEmpty).to_string(),
                vec![ValidationDetail::new("email", ValidationReason::Required)],
            ));
        }

        if self.password.is_empty() {
            return Err(AppError::BadRequestWithDetails(
                ErrorCode::UserPasswordEmpty,
                translate(locale, MessageKey::PasswordCannotBeEmpty).to_string(),
                vec![ValidationDetail::new(
                    "password",
                    ValidationReason::Required,
                )],
            ));
        }

        Ok(())
    }
}

/// Refresh 请求。
///
/// 用于 `POST /auth/refresh`。
/// 调用方需要提交一个尚未过期且未被撤销的 `refresh_token`。
#[derive(Debug, Deserialize)]
pub struct RefreshRequest {
    /// 可用于续签访问令牌的 refresh token。
    pub refresh_token: String,
}

impl RefreshRequest {
    /// 校验刷新令牌请求。
    ///
    /// 当前仅校验 `refresh_token` 非空，合法性与状态由会话域服务继续校验。
    pub fn validate(&self, locale: Locale) -> Result<(), AppError> {
        if self.refresh_token.trim().is_empty() {
            return Err(AppError::BadRequestWithDetails(
                ErrorCode::AuthMissingRefreshToken,
                translate(locale, MessageKey::MissingRefreshToken).to_string(),
                vec![ValidationDetail::new(
                    "refresh_token",
                    ValidationReason::Required,
                )],
            ));
        }

        Ok(())
    }
}

/// Logout 请求。
///
/// 用于 `POST /auth/logout`。
/// 该接口通过撤销传入的 `refresh_token` 来使对应会话失效。
#[derive(Debug, Deserialize)]
pub struct LogoutRequest {
    /// 需要撤销的 refresh token。
    pub refresh_token: String,
}

impl LogoutRequest {
    /// 校验登出请求。
    ///
    /// 当前仅校验 `refresh_token` 非空。
    pub fn validate(&self, locale: Locale) -> Result<(), AppError> {
        if self.refresh_token.trim().is_empty() {
            return Err(AppError::BadRequestWithDetails(
                ErrorCode::AuthMissingRefreshToken,
                translate(locale, MessageKey::MissingRefreshToken).to_string(),
                vec![ValidationDetail::new(
                    "refresh_token",
                    ValidationReason::Required,
                )],
            ));
        }

        Ok(())
    }
}

/// Token 响应。
///
/// 该结构表示一次认证成功后返回的一组令牌：
/// - `access_token`：短期访问令牌，供受保护接口放入 `Authorization` 请求头
/// - `refresh_token`：长期刷新令牌，供续签接口轮换新令牌对
/// - `expires_in` / `refresh_expires_in`：相对当前响应时间的剩余秒数
#[derive(Debug, Serialize)]
pub struct TokenResponse {
    /// 短期访问令牌。
    pub access_token: String,
    /// 长期刷新令牌。
    pub refresh_token: String,
    /// 令牌类型，当前固定为 `Bearer`。
    pub token_type: String,
    /// access token 过期秒数。
    pub expires_in: i64,
    /// refresh token 过期秒数。
    pub refresh_expires_in: i64,
}

/// 登录响应。
///
/// 同时用于登录和刷新接口的成功响应体。
#[derive(Debug, Serialize)]
pub struct LoginResponse {
    /// 本次登录签发的令牌对。
    pub token: TokenResponse,
    /// 当前登录用户资料。
    pub user: UserResponse,
}
