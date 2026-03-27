use app_foundation::{ApiResponse, AppError};
use axum::{
    extract::{Extension, Path, State},
    Json,
};
use uuid::Uuid;

use crate::{
    modules::{
        auth::dto::{
            LoginRequest, LoginResponse, LogoutRequest, RefreshRequest, RegisterRequest,
            RevokeAllSessionsResponse, SessionResponse,
        },
        auth::service,
        identity,
        rbac::context::AccessContext,
        user::model::UserResponse,
    },
    state::AppState,
};

/// 处理用户注册请求。
pub async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<ApiResponse<UserResponse>>, AppError> {
    let user = identity::service::register_user(&state, req).await?;
    Ok(Json(ApiResponse::ok(user)))
}

/// 处理登录请求。
pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<ApiResponse<LoginResponse>>, AppError> {
    let result = service::login(&state, req).await?;
    Ok(Json(ApiResponse::ok(result)))
}

/// 处理 refresh token 续签请求。
pub async fn refresh(
    State(state): State<AppState>,
    Json(req): Json<RefreshRequest>,
) -> Result<Json<ApiResponse<LoginResponse>>, AppError> {
    let result = service::refresh(&state, req).await?;
    Ok(Json(ApiResponse::ok(result)))
}

/// 处理登出请求并撤销 refresh token。
pub async fn logout(
    State(state): State<AppState>,
    Json(req): Json<LogoutRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    service::logout(&state, req).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "logged_out": true
    }))))
}

/// 列出当前用户的认证会话。
pub async fn list_sessions(
    State(state): State<AppState>,
    Extension(current_user): Extension<AccessContext>,
) -> Result<Json<ApiResponse<Vec<SessionResponse>>>, AppError> {
    let sessions = service::list_sessions(&state, current_user.user_id).await?;
    Ok(Json(ApiResponse::ok(sessions)))
}

/// 撤销当前用户的指定会话。
pub async fn revoke_session(
    State(state): State<AppState>,
    Extension(current_user): Extension<AccessContext>,
    Path(session_id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    service::revoke_session(&state, current_user.user_id, session_id).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "revoked": true
    }))))
}

/// 撤销当前用户的全部会话。
pub async fn revoke_all_sessions(
    State(state): State<AppState>,
    Extension(current_user): Extension<AccessContext>,
) -> Result<Json<ApiResponse<RevokeAllSessionsResponse>>, AppError> {
    let result = service::revoke_all_sessions(&state, current_user.user_id).await?;
    Ok(Json(ApiResponse::ok(result)))
}
