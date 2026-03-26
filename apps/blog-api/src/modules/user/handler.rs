use app_foundation::{ApiResponse, AppError};
use axum::{
    extract::{Extension, Path, Query, State},
    Json,
};
use uuid::Uuid;

use crate::{
    modules::{
        rbac::authorization,
        rbac::context::AccessContext,
        rbac::keys::PermissionKey,
        user::{
            dto::{CreateUserRequest, UpdateUserRequest, UserListQuery, UserListResponse},
            model::UserResponse,
            service,
        },
    },
    state::AppState,
};

pub async fn create_user(
    State(state): State<AppState>,
    Extension(current_user): Extension<AccessContext>,
    Json(req): Json<CreateUserRequest>,
) -> Result<Json<ApiResponse<UserResponse>>, AppError> {
    authorization::require_scope(&state, &current_user, PermissionKey::USERS_WRITE)?;
    let user = service::create_user(&state, req).await?;
    Ok(Json(ApiResponse::ok(user)))
}

pub async fn get_user(
    State(state): State<AppState>,
    Extension(current_user): Extension<AccessContext>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<UserResponse>>, AppError> {
    authorization::require_scope(&state, &current_user, PermissionKey::USERS_READ)?;
    let user = service::get_user(&state, id).await?;
    Ok(Json(ApiResponse::ok(user)))
}

pub async fn me(
    State(state): State<AppState>,
    Extension(current_user): Extension<AccessContext>,
) -> Result<Json<ApiResponse<UserResponse>>, AppError> {
    authorization::require_scope(&state, &current_user, PermissionKey::USERS_READ)?;
    let user = service::me(&state, current_user.user_id).await?;

    Ok(Json(ApiResponse::ok(user)))
}

pub async fn list_users(
    State(state): State<AppState>,
    Extension(current_user): Extension<AccessContext>,
    Query(query): Query<UserListQuery>,
) -> Result<Json<ApiResponse<UserListResponse>>, AppError> {
    authorization::require_scope(&state, &current_user, PermissionKey::USERS_READ)?;
    let normalized = query.normalize(1, 10, 100);

    let result = service::list_users(&state, normalized.page, normalized.page_size).await?;
    Ok(Json(ApiResponse::ok(result)))
}

pub async fn update_user(
    State(state): State<AppState>,
    Extension(current_user): Extension<AccessContext>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateUserRequest>,
) -> Result<Json<ApiResponse<UserResponse>>, AppError> {
    authorization::require_scope(&state, &current_user, PermissionKey::USERS_WRITE)?;
    let user = service::update_user(&state, id, req).await?;
    Ok(Json(ApiResponse::ok(user)))
}

pub async fn delete_user(
    State(state): State<AppState>,
    Extension(current_user): Extension<AccessContext>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    authorization::require_scope(&state, &current_user, PermissionKey::USERS_WRITE)?;
    service::delete_user(&state, id).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "deleted": true
    }))))
}
