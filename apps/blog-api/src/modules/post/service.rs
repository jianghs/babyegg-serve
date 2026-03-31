use app_foundation::{
    i18n::{translate, MessageKey},
    AppError, ErrorCode, PageResponse,
};
use tracing::info;
use uuid::Uuid;

use crate::{
    db::post_repo,
    modules::post::{
        dto::{CreatePostRequest, NormalizedPostListQuery, PostListResponse, UpdatePostRequest},
        model::PostResponse,
    },
    state::AppState,
};

/// 创建博文资源。
pub async fn create_post(
    state: &AppState,
    author_id: Uuid,
    req: CreatePostRequest,
) -> Result<PostResponse, AppError> {
    let locale = state.config.base.default_locale;
    req.validate(locale)?;

    let post = post_repo::create_post(
        &state.db,
        author_id,
        &req.title,
        &req.slug,
        &req.content_md,
        req.published.unwrap_or(false),
    )
    .await
    .map_err(|_| {
        AppError::InternalWithMessage(
            translate(locale, MessageKey::InternalServerError).to_string(),
        )
    })?;

    info!(post_id = %post.id, slug = %post.slug, author_id = %post.author_id, "post created");

    Ok(post.into())
}

/// 获取单个博文资源。
pub async fn get_post(state: &AppState, id: Uuid) -> Result<PostResponse, AppError> {
    let locale = state.config.base.default_locale;
    let post = post_repo::get_post(&state.db, id).await.map_err(|_| {
        AppError::InternalWithMessage(
            translate(locale, MessageKey::InternalServerError).to_string(),
        )
    })?;

    post.map(PostResponse::from)
        .ok_or(AppError::NotFoundWithCode(
            ErrorCode::NotFound,
            translate(locale, MessageKey::NotFound).to_string(),
        ))
}

/// 按 slug 获取单个博文资源。
pub async fn get_post_by_slug(state: &AppState, slug: &str) -> Result<PostResponse, AppError> {
    let locale = state.config.base.default_locale;
    let post = post_repo::get_post_by_slug(&state.db, slug)
        .await
        .map_err(|_| {
            AppError::InternalWithMessage(
                translate(locale, MessageKey::InternalServerError).to_string(),
            )
        })?;

    post.map(PostResponse::from)
        .ok_or(AppError::NotFoundWithCode(
            ErrorCode::NotFound,
            translate(locale, MessageKey::NotFound).to_string(),
        ))
}

/// 查询分页博文列表。
pub async fn list_posts(
    state: &AppState,
    query: NormalizedPostListQuery,
) -> Result<PostListResponse, AppError> {
    let locale = state.config.base.default_locale;
    let items = post_repo::list_posts(&state.db, &query)
        .await
        .map_err(|_| {
            AppError::InternalWithMessage(
                translate(locale, MessageKey::InternalServerError).to_string(),
            )
        })?
        .into_iter()
        .map(PostResponse::from)
        .collect();
    let total = post_repo::count_posts(&state.db, query.filter.as_deref(), query.published)
        .await
        .map_err(|_| {
            AppError::InternalWithMessage(
                translate(locale, MessageKey::InternalServerError).to_string(),
            )
        })?;

    Ok(PageResponse::new(items, query.page, query.page_size, total))
}

/// 更新博文资源。
pub async fn update_post(
    state: &AppState,
    id: Uuid,
    req: UpdatePostRequest,
) -> Result<PostResponse, AppError> {
    let locale = state.config.base.default_locale;
    req.validate(locale)?;

    let post = post_repo::update_post(
        &state.db,
        id,
        &req.title,
        &req.slug,
        &req.content_md,
        req.published,
    )
    .await
    .map_err(|_| {
        AppError::InternalWithMessage(
            translate(locale, MessageKey::InternalServerError).to_string(),
        )
    })?;

    post.map(PostResponse::from)
        .ok_or(AppError::NotFoundWithCode(
            ErrorCode::NotFound,
            translate(locale, MessageKey::NotFound).to_string(),
        ))
}

/// 删除博文资源。
pub async fn delete_post(state: &AppState, id: Uuid) -> Result<(), AppError> {
    let locale = state.config.base.default_locale;
    let deleted = post_repo::delete_post(&state.db, id).await.map_err(|_| {
        AppError::InternalWithMessage(
            translate(locale, MessageKey::InternalServerError).to_string(),
        )
    })?;

    if !deleted {
        return Err(AppError::NotFoundWithCode(
            ErrorCode::NotFound,
            translate(locale, MessageKey::NotFound).to_string(),
        ));
    }

    Ok(())
}
