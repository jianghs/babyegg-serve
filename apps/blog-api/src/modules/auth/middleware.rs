use app_foundation::i18n::{translate, MessageKey};
use app_foundation::{AppError, ErrorCode, ValidationDetail, ValidationReason};
use axum::{
    extract::Request, extract::State, http::header::AUTHORIZATION, middleware::Next,
    response::Response,
};

use crate::{modules::rbac::context::AccessContext, state::AppState};

use super::jwt;

/// 统一 Bearer Token 鉴权中间件。
///
/// 典型请求头格式：
/// - `Authorization: Bearer <access_token>`
///
/// 处理流程：
/// - 校验 `Authorization` 请求头存在
/// - 校验前缀必须是 `Bearer `
/// - 校验 JWT 签名与过期时间
/// - 将解析后的 [`AccessContext`]
///   注入到 request extensions
///
/// 成功后下游 handler 可通过 `Extension<AccessContext>` 直接读取当前访问主体。
pub async fn require_auth(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let locale = state.config.base.default_locale;

    let authorization = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok());

    let auth_header = authorization.ok_or(AppError::BadRequestWithDetails(
        ErrorCode::AuthMissingAuthorizationHeader,
        translate(locale, MessageKey::MissingAuthorizationHeader).to_string(),
        vec![ValidationDetail::new(
            "authorization",
            ValidationReason::Required,
        )],
    ))?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(AppError::BadRequestWithDetails(
            ErrorCode::AuthInvalidAuthorizationHeader,
            translate(locale, MessageKey::InvalidAuthorizationHeader).to_string(),
            vec![ValidationDetail::new(
                "authorization",
                ValidationReason::InvalidFormat,
            )],
        ))?;

    let claims = jwt::verify_token(token, &state.config.auth.jwt_secret).map_err(|_| {
        AppError::BadRequestWithCode(
            ErrorCode::AuthInvalidToken,
            translate(locale, MessageKey::InvalidToken).to_string(),
        )
    })?;

    let access_context = AccessContext::try_from(claims).map_err(|_| {
        AppError::BadRequestWithCode(
            ErrorCode::AuthInvalidTokenSubject,
            translate(locale, MessageKey::InvalidTokenSubject).to_string(),
        )
    })?;

    req.extensions_mut().insert(access_context);
    Ok(next.run(req).await)
}
