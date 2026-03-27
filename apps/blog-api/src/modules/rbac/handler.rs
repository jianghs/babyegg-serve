use app_foundation::{ApiResponse, AppError};
use axum::{
    extract::{Extension, Path, State},
    Json,
};
use uuid::Uuid;

use crate::{
    modules::rbac::{
        authorization,
        context::AccessContext,
        dto::{AssignUserRoleRequest, PermissionResponse, RoleResponse, UserAccessResponse},
        keys::RoleKey,
        service,
    },
    state::AppState,
};

/// 处理查询角色定义列表请求。
pub async fn list_roles(
    State(state): State<AppState>,
    Extension(current_user): Extension<AccessContext>,
) -> Result<Json<ApiResponse<Vec<RoleResponse>>>, AppError> {
    authorization::require_role(&state, &current_user, RoleKey::ADMIN)?;
    let roles = service::list_roles(&state).await?;
    Ok(Json(ApiResponse::ok(roles)))
}

/// 处理查询权限定义列表请求。
pub async fn list_permissions(
    State(state): State<AppState>,
    Extension(current_user): Extension<AccessContext>,
) -> Result<Json<ApiResponse<Vec<PermissionResponse>>>, AppError> {
    authorization::require_role(&state, &current_user, RoleKey::ADMIN)?;
    let permissions = service::list_permissions(&state).await?;
    Ok(Json(ApiResponse::ok(permissions)))
}

/// 处理查询用户访问上下文请求。
pub async fn get_user_access(
    State(state): State<AppState>,
    Extension(current_user): Extension<AccessContext>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<ApiResponse<UserAccessResponse>>, AppError> {
    authorization::require_role(&state, &current_user, RoleKey::ADMIN)?;
    let access = service::get_user_access(&state, user_id).await?;
    Ok(Json(ApiResponse::ok(access)))
}

/// 处理给用户分配角色请求。
pub async fn assign_user_role(
    State(state): State<AppState>,
    Extension(current_user): Extension<AccessContext>,
    Path(user_id): Path<Uuid>,
    Json(req): Json<AssignUserRoleRequest>,
) -> Result<Json<ApiResponse<UserAccessResponse>>, AppError> {
    authorization::require_role(&state, &current_user, RoleKey::ADMIN)?;
    let access = service::assign_user_role(&state, user_id, req).await?;
    Ok(Json(ApiResponse::ok(access)))
}
