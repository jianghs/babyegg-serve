use app_foundation::i18n::{translate, MessageKey};
use app_foundation::{AppError, ErrorCode, ValidationDetail};
use axum::{
    extract::Request, extract::State, http::header::AUTHORIZATION, middleware::Next,
    response::Response,
};
use uuid::Uuid;

use crate::state::AppState;

use super::{current_user::CurrentUser, jwt};

/// 统一 Bearer Token 鉴权中间件。
///
/// 成功后会在 request extensions 中注入 CurrentUser。
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
        vec![ValidationDetail::new("authorization", "required")],
    ))?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(AppError::BadRequestWithDetails(
            ErrorCode::AuthInvalidAuthorizationHeader,
            translate(locale, MessageKey::InvalidAuthorizationHeader).to_string(),
            vec![ValidationDetail::new("authorization", "invalid_format")],
        ))?;

    let claims = jwt::verify_token(token, &state.config.jwt_secret).map_err(|_| {
        AppError::BadRequestWithCode(
            ErrorCode::AuthInvalidToken,
            translate(locale, MessageKey::InvalidToken).to_string(),
        )
    })?;

    let user_id = claims.sub.parse::<Uuid>().map_err(|_| {
        AppError::BadRequestWithCode(
            ErrorCode::AuthInvalidTokenSubject,
            translate(locale, MessageKey::InvalidTokenSubject).to_string(),
        )
    })?;

    req.extensions_mut().insert(CurrentUser { user_id });
    Ok(next.run(req).await)
}
