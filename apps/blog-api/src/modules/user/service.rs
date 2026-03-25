use app_foundation::{
    error::AppError,
    i18n::{translate, MessageKey},
    PageResponse,
};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};
use tracing::info;
use uuid::Uuid;

use crate::{
    db::user_repo,
    modules::user::{
        dto::{CreateUserRequest, UserListResponse},
        model::UserResponse,
    },
    state::AppState,
};

/// 创建用户。
pub async fn create_user(
    state: &AppState,
    req: CreateUserRequest,
) -> Result<UserResponse, AppError> {
    let locale = state.config.base.default_locale;

    if req.name.trim().is_empty() {
        return Err(AppError::BadRequest(
            translate(
                state.config.base.default_locale,
                MessageKey::NameCannotBeEmpty,
            )
            .to_string(),
        ));
    }

    if req.email.trim().is_empty() {
        return Err(AppError::BadRequest(
            translate(
                state.config.base.default_locale,
                MessageKey::EmailCannotBeEmpty,
            )
            .to_string(),
        ));
    }

    if req.password.len() < 6 {
        return Err(AppError::BadRequest(
            translate(
                state.config.base.default_locale,
                MessageKey::PasswordTooShort,
            )
            .to_string(),
        ));
    }

    let existing = user_repo::get_user_by_email(&state.db, &req.email)
        .await
        .map_err(|_| {
            AppError::InternalWithMessage(
                translate(locale, MessageKey::InternalServerError).to_string(),
            )
        })?;

    if existing.is_some() {
        return Err(AppError::BadRequest(
            translate(
                state.config.base.default_locale,
                MessageKey::EmailAlreadyExists,
            )
            .to_string(),
        ));
    }

    let salt = SaltString::generate(&mut OsRng);
    let password_hash = Argon2::default()
        .hash_password(req.password.as_bytes(), &salt)
        .map_err(|_| {
            AppError::InternalWithMessage(
                translate(locale, MessageKey::InternalServerError).to_string(),
            )
        })?
        .to_string();

    let user = user_repo::create_user(&state.db, &req.name, &req.email, &password_hash)
        .await
        .map_err(|_| {
            AppError::InternalWithMessage(
                translate(locale, MessageKey::InternalServerError).to_string(),
            )
        })?;

    info!(
        message = translate(state.config.base.default_locale, MessageKey::RequestReceived),
        user_id = %user.id,
        email = %user.email
    );

    Ok(user.into())
}

/// 获取单个用户。
pub async fn get_user(state: &AppState, id: Uuid) -> Result<UserResponse, AppError> {
    let locale = state.config.base.default_locale;

    let user = user_repo::get_user(&state.db, id)
        .await
        .map_err(|_| {
            AppError::InternalWithMessage(
                translate(locale, MessageKey::InternalServerError).to_string(),
            )
        })?
        .ok_or(AppError::NotFoundWithMessage(
            translate(locale, MessageKey::NotFound).to_string(),
        ))?;

    Ok(user.into())
}

/// 获取当前登录用户。
pub async fn me(state: &AppState, user_id: Uuid) -> Result<UserResponse, AppError> {
    let locale = state.config.base.default_locale;

    let user = user_repo::get_user(&state.db, user_id)
        .await
        .map_err(|_| {
            AppError::InternalWithMessage(
                translate(locale, MessageKey::InternalServerError).to_string(),
            )
        })?
        .ok_or(AppError::NotFoundWithMessage(
            translate(locale, MessageKey::NotFound).to_string(),
        ))?;

    Ok(user.into())
}

/// 查询用户列表。
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

/// 更新用户名称。
pub async fn update_user(
    state: &AppState,
    id: Uuid,
    name: String,
) -> Result<UserResponse, AppError> {
    let locale = state.config.base.default_locale;

    if name.trim().is_empty() {
        return Err(AppError::BadRequest(
            translate(
                state.config.base.default_locale,
                MessageKey::NameCannotBeEmpty,
            )
            .to_string(),
        ));
    }

    let user = user_repo::update_user_name(&state.db, id, &name)
        .await
        .map_err(|_| {
            AppError::InternalWithMessage(
                translate(locale, MessageKey::InternalServerError).to_string(),
            )
        })?
        .ok_or(AppError::NotFoundWithMessage(
            translate(locale, MessageKey::NotFound).to_string(),
        ))?;

    Ok(user.into())
}

/// 删除用户。
pub async fn delete_user(state: &AppState, id: Uuid) -> Result<(), AppError> {
    let locale = state.config.base.default_locale;

    let deleted = user_repo::delete_user(&state.db, id).await.map_err(|_| {
        AppError::InternalWithMessage(
            translate(locale, MessageKey::InternalServerError).to_string(),
        )
    })?;

    if !deleted {
        return Err(AppError::NotFoundWithMessage(
            translate(locale, MessageKey::NotFound).to_string(),
        ));
    }

    Ok(())
}
