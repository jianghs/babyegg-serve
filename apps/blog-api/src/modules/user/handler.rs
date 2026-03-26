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

/// 处理创建用户请求。
///
/// 对应 `POST /users`。
/// 调用前要求当前访问主体具备 [`PermissionKey::USERS_WRITE`](crate::modules::rbac::keys::PermissionKey::USERS_WRITE)。
pub async fn create_user(
    State(state): State<AppState>,
    Extension(current_user): Extension<AccessContext>,
    Json(req): Json<CreateUserRequest>,
) -> Result<Json<ApiResponse<UserResponse>>, AppError> {
    authorization::require_scope(&state, &current_user, PermissionKey::USERS_WRITE)?;
    let user = service::create_user(&state, req).await?;
    Ok(Json(ApiResponse::ok(user)))
}

/// 处理获取单个用户请求。
///
/// 对应 `GET /users/{id}`。
/// 调用前要求当前访问主体具备 [`PermissionKey::USERS_READ`](crate::modules::rbac::keys::PermissionKey::USERS_READ)。
pub async fn get_user(
    State(state): State<AppState>,
    Extension(current_user): Extension<AccessContext>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<UserResponse>>, AppError> {
    authorization::require_scope(&state, &current_user, PermissionKey::USERS_READ)?;
    let user = service::get_user(&state, id).await?;
    Ok(Json(ApiResponse::ok(user)))
}

/// 处理查询当前用户资料请求。
///
/// 对应 `GET /users/me`。
/// 该接口从认证中间件注入的 [`AccessContext`]
/// 中读取当前用户 ID，而不是从路径参数获取。
pub async fn me(
    State(state): State<AppState>,
    Extension(current_user): Extension<AccessContext>,
) -> Result<Json<ApiResponse<UserResponse>>, AppError> {
    authorization::require_scope(&state, &current_user, PermissionKey::USERS_READ)?;
    let user = service::me(&state, current_user.user_id).await?;

    Ok(Json(ApiResponse::ok(user)))
}

/// 处理分页查询用户列表请求。
///
/// 对应 `GET /users`。
/// 查询参数会先归一化为默认 `page = 1`、`page_size = 10`，并将页容量限制在 `100` 以内。
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

/// 处理更新用户请求。
///
/// 对应 `PUT /users/{id}`。
/// 当前仅支持更新用户名，邮箱、密码与角色不在该接口职责范围内。
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

/// 处理删除用户请求。
///
/// 对应 `DELETE /users/{id}`。
/// 成功时返回 `{ "deleted": true }`。
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
