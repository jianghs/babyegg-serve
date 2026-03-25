use app_foundation::{ApiResponse, AppError};
use axum::{
    extract::{Path, Query, State},
    http::HeaderMap,
    Json,
};
use uuid::Uuid;

use crate::{
    modules::{
        auth::handler::parse_bearer_user_id,
        user::{
            dto::{CreateUserRequest, PaginationQuery, UpdateUserRequest, UserListResponse},
            model::UserResponse,
            service,
        },
    },
    state::AppState,
};

pub async fn create_user(
    State(state): State<AppState>,
    Json(req): Json<CreateUserRequest>,
) -> Result<Json<ApiResponse<UserResponse>>, AppError> {
    let user = service::create_user(&state, req).await?;
    Ok(Json(ApiResponse::ok(user)))
}

pub async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<UserResponse>>, AppError> {
    let user = service::get_user(&state, id).await?;
    Ok(Json(ApiResponse::ok(user)))
}

pub async fn me(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<UserResponse>>, AppError> {
    let authorization = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok());

    let user_id = parse_bearer_user_id(authorization, &state)?;
    let user = service::me(&state, user_id).await?;

    Ok(Json(ApiResponse::ok(user)))
}

pub async fn list_users(
    State(state): State<AppState>,
    Query(query): Query<PaginationQuery>,
) -> Result<Json<ApiResponse<UserListResponse>>, AppError> {
    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(10);

    let result = service::list_users(&state, page, page_size).await?;
    Ok(Json(ApiResponse::ok(result)))
}

pub async fn update_user(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateUserRequest>,
) -> Result<Json<ApiResponse<UserResponse>>, AppError> {
    let user = service::update_user(&state, id, req).await?;
    Ok(Json(ApiResponse::ok(user)))
}

pub async fn delete_user(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    service::delete_user(&state, id).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "deleted": true
    }))))
}
