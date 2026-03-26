use app_foundation::{ApiResponse, AppError};
use axum::{extract::State, Json};

use crate::{
    modules::{
        auth::dto::{LoginRequest, LoginResponse, LogoutRequest, RefreshRequest, RegisterRequest},
        auth::service,
        identity,
        user::model::UserResponse,
    },
    state::AppState,
};

pub async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<ApiResponse<UserResponse>>, AppError> {
    let user = identity::service::register_user(&state, req).await?;
    Ok(Json(ApiResponse::ok(user)))
}

pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<ApiResponse<LoginResponse>>, AppError> {
    let result = service::login(&state, req).await?;
    Ok(Json(ApiResponse::ok(result)))
}

pub async fn refresh(
    State(state): State<AppState>,
    Json(req): Json<RefreshRequest>,
) -> Result<Json<ApiResponse<LoginResponse>>, AppError> {
    let result = service::refresh(&state, req).await?;
    Ok(Json(ApiResponse::ok(result)))
}

pub async fn logout(
    State(state): State<AppState>,
    Json(req): Json<LogoutRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    service::logout(&state, req).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "logged_out": true
    }))))
}
