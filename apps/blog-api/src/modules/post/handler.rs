use app_foundation::{ApiResponse, AppError};
use axum::{
    extract::{Extension, Path, Query, State},
    Json,
};
use uuid::Uuid;

use crate::{
    modules::{
        post::{
            dto::{CreatePostRequest, PostListQuery, PostListResponse, UpdatePostRequest},
            model::PostResponse,
            service,
        },
        rbac::authorization,
        rbac::context::AccessContext,
        rbac::keys::PermissionKey,
    },
    state::AppState,
};

/// 处理创建博文请求。
pub async fn create_post(
    State(state): State<AppState>,
    Extension(current_user): Extension<AccessContext>,
    Json(req): Json<CreatePostRequest>,
) -> Result<Json<ApiResponse<PostResponse>>, AppError> {
    authorization::require_scope(&state, &current_user, PermissionKey::POSTS_WRITE)?;
    let post = service::create_post(&state, current_user.user_id, req).await?;
    Ok(Json(ApiResponse::ok(post)))
}

/// 处理获取单个博文请求。
pub async fn get_post(
    State(state): State<AppState>,
    Extension(current_user): Extension<AccessContext>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<PostResponse>>, AppError> {
    authorization::require_scope(&state, &current_user, PermissionKey::POSTS_READ)?;
    let post = service::get_post(&state, id).await?;
    Ok(Json(ApiResponse::ok(post)))
}

/// 处理按 slug 获取单个博文请求。
pub async fn get_post_by_slug(
    State(state): State<AppState>,
    Extension(current_user): Extension<AccessContext>,
    Path(slug): Path<String>,
) -> Result<Json<ApiResponse<PostResponse>>, AppError> {
    authorization::require_scope(&state, &current_user, PermissionKey::POSTS_READ)?;
    let post = service::get_post_by_slug(&state, slug.trim()).await?;
    Ok(Json(ApiResponse::ok(post)))
}

/// 处理分页查询博文列表请求。
pub async fn list_posts(
    State(state): State<AppState>,
    Extension(current_user): Extension<AccessContext>,
    Query(query): Query<PostListQuery>,
) -> Result<Json<ApiResponse<PostListResponse>>, AppError> {
    authorization::require_scope(&state, &current_user, PermissionKey::POSTS_READ)?;
    let normalized = query.normalize(state.config.base.default_locale, 1, 10, 100)?;
    let posts = service::list_posts(&state, normalized).await?;
    Ok(Json(ApiResponse::ok(posts)))
}

/// 处理更新博文请求。
pub async fn update_post(
    State(state): State<AppState>,
    Extension(current_user): Extension<AccessContext>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdatePostRequest>,
) -> Result<Json<ApiResponse<PostResponse>>, AppError> {
    authorization::require_scope(&state, &current_user, PermissionKey::POSTS_WRITE)?;
    let post = service::update_post(&state, id, req).await?;
    Ok(Json(ApiResponse::ok(post)))
}

/// 处理删除博文请求。
pub async fn delete_post(
    State(state): State<AppState>,
    Extension(current_user): Extension<AccessContext>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    authorization::require_scope(&state, &current_user, PermissionKey::POSTS_WRITE)?;
    service::delete_post(&state, id).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "deleted": true
    }))))
}
