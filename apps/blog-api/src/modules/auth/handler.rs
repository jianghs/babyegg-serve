use app_foundation::{ApiResponse, AppError};
use axum::{extract::State, Json};

use crate::{
    modules::{
        auth::dto::{LoginRequest, LoginResponse, RegisterRequest},
        auth::service,
        user::model::UserResponse,
    },
    state::AppState,
};

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
