use app_foundation::{
    error::AppError,
    i18n::{translate, MessageKey},
    ErrorCode, PageResponse,
};
use tracing::info;
use uuid::Uuid;

use crate::{
    db::user_repo,
    modules::{
        identity,
        user::{
            dto::{CreateUserRequest, UpdateUserRequest, UserListResponse},
            model::UserResponse,
        },
    },
    state::AppState,
};

/// 创建用户资源。
///
/// 该函数复用身份域建档逻辑，因此会继承邮箱唯一性校验与密码哈希规则。
pub async fn create_user(
    state: &AppState,
    req: CreateUserRequest,
) -> Result<UserResponse, AppError> {
    let user = identity::service::create_user(state, req).await?;

    info!(
        message = translate(state.config.base.default_locale, MessageKey::RequestReceived),
        user_id = %user.id,
        email = %user.email
    );

    Ok(user)
}

/// 按用户 ID 获取单个用户资源。
///
/// 当用户不存在时返回 [`ErrorCode::NotFound`]。
pub async fn get_user(state: &AppState, id: Uuid) -> Result<UserResponse, AppError> {
    let locale = state.config.base.default_locale;

    let user = identity::service::get_user(state, id)
        .await?
        .ok_or(AppError::NotFoundWithCode(
            ErrorCode::NotFound,
            translate(locale, MessageKey::NotFound).to_string(),
        ))?;

    Ok(user.into())
}

/// 获取当前登录用户资料。
///
/// 当认证成功但数据库中对应用户不存在时，同样返回 `NotFound`。
pub async fn me(state: &AppState, user_id: Uuid) -> Result<UserResponse, AppError> {
    let locale = state.config.base.default_locale;

    let user =
        identity::service::get_user(state, user_id)
            .await?
            .ok_or(AppError::NotFoundWithCode(
                ErrorCode::NotFound,
                translate(locale, MessageKey::NotFound).to_string(),
            ))?;

    Ok(user.into())
}

/// 查询分页用户列表。
///
/// 传入参数会再次做边界保护：
/// - `page` 最小为 `1`
/// - `page_size` 范围被限制在 `1..=100`
pub async fn list_users(
    state: &AppState,
    page: i64,
    page_size: i64,
) -> Result<UserListResponse, AppError> {
    let locale = state.config.base.default_locale;

    let page = page.max(1);
    let page_size = page_size.clamp(1, 100);

    let items = user_repo::list_users(&state.db, page, page_size)
        .await
        .map_err(|_| {
            AppError::InternalWithMessage(
                translate(locale, MessageKey::InternalServerError).to_string(),
            )
        })?
        .into_iter()
        .map(UserResponse::from)
        .collect();

    let total = user_repo::count_users(&state.db).await.map_err(|_| {
        AppError::InternalWithMessage(
            translate(locale, MessageKey::InternalServerError).to_string(),
        )
    })?;

    Ok(PageResponse::new(items, page, page_size, total))
}

/// 更新指定用户的显示名。
///
/// 当目标用户不存在时返回 [`ErrorCode::NotFound`]。
pub async fn update_user(
    state: &AppState,
    id: Uuid,
    req: UpdateUserRequest,
) -> Result<UserResponse, AppError> {
    let locale = state.config.base.default_locale;
    req.validate(locale)?;

    let user = user_repo::update_user_name(&state.db, id, &req.name)
        .await
        .map_err(|_| {
            AppError::InternalWithMessage(
                translate(locale, MessageKey::InternalServerError).to_string(),
            )
        })?
        .ok_or(AppError::NotFoundWithCode(
            ErrorCode::NotFound,
            translate(locale, MessageKey::NotFound).to_string(),
        ))?;

    Ok(user.into())
}

/// 删除指定用户。
///
/// 删除操作是幂等语义上的“按结果反馈”：
/// - 删除成功返回 `Ok(())`
/// - 目标不存在返回 `NotFound`
pub async fn delete_user(state: &AppState, id: Uuid) -> Result<(), AppError> {
    let locale = state.config.base.default_locale;

    let deleted = user_repo::delete_user(&state.db, id).await.map_err(|_| {
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
