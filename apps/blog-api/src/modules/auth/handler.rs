use app_foundation::i18n::{translate, MessageKey};
use app_foundation::{ApiResponse, AppError, ErrorCode, ValidationDetail};
use axum::{extract::State, Json};
use uuid::Uuid;

use crate::{
    modules::{
        auth::{
            dto::{LoginRequest, LoginResponse, RegisterRequest},
            jwt, service,
        },
        user::model::UserResponse,
    },
    state::AppState,
};

/// 从 Authorization 头中解析当前用户 ID。
pub fn parse_bearer_user_id(
    authorization: Option<&str>,
    state: &AppState,
) -> Result<Uuid, AppError> {
    let locale = state.config.base.default_locale;

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

    claims.sub.parse::<Uuid>().map_err(|_| {
        AppError::BadRequestWithCode(
            ErrorCode::AuthInvalidTokenSubject,
            translate(locale, MessageKey::InvalidTokenSubject).to_string(),
        )
    })
}

pub async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<ApiResponse<UserResponse>>, AppError> {
    let user = service::register(&state, req).await?;
    Ok(Json(ApiResponse::ok(user)))
}

pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<ApiResponse<LoginResponse>>, AppError> {
    let result = service::login(&state, req).await?;
    Ok(Json(ApiResponse::ok(result)))
}
