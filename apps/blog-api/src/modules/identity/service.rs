//! 身份域服务负责用户建档、密码哈希与凭证校验。

use app_foundation::i18n::{translate, MessageKey};
use app_foundation::{AppError, ErrorCode};
use argon2::password_hash::rand_core::OsRng;
use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use uuid::Uuid;

use crate::{
    db::user_repo,
    modules::{
        auth::dto::RegisterRequest,
        identity::{
            dto::CreateIdentityUser,
            model::{User, UserResponse},
        },
        rbac::{keys::RoleKey, service as rbac_service},
        user::dto::CreateUserRequest,
    },
    state::AppState,
};

/// 创建普通注册用户，并在成功后自动分配默认 `user` 角色。
pub async fn register_user(
    state: &AppState,
    req: RegisterRequest,
) -> Result<UserResponse, AppError> {
    let locale = state.config.base.default_locale;
    let input = CreateIdentityUser::from(req);
    input.validate(locale)?;

    let user = create_user_record(state, &input).await?;
    rbac_service::assign_role_or_fail(state, user.id, RoleKey::USER).await?;

    Ok(user.into())
}

/// 创建用户记录，但不附带认证会话或默认角色分配。
pub async fn create_user(
    state: &AppState,
    req: CreateUserRequest,
) -> Result<UserResponse, AppError> {
    let locale = state.config.base.default_locale;
    let input = CreateIdentityUser::from(req);
    input.validate(locale)?;

    let user = create_user_record(state, &input).await?;
    Ok(user.into())
}

/// 校验邮箱与密码，并返回完整用户实体供认证流程继续使用。
pub async fn verify_credentials(
    state: &AppState,
    email: &str,
    password: &str,
) -> Result<User, AppError> {
    let locale = state.config.base.default_locale;

    let user = user_repo::get_user_by_email(&state.db, email)
        .await
        .map_err(|_| {
            AppError::InternalWithMessage(
                translate(locale, MessageKey::InternalServerError).to_string(),
            )
        })?
        .ok_or(AppError::BadRequestWithCode(
            ErrorCode::AuthInvalidCredentials,
            translate(locale, MessageKey::InvalidEmailOrPassword).to_string(),
        ))?;

    let parsed_hash = PasswordHash::new(&user.password_hash).map_err(|_| {
        AppError::InternalWithMessage(
            translate(locale, MessageKey::InternalServerError).to_string(),
        )
    })?;

    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .map_err(|_| {
            AppError::BadRequestWithCode(
                ErrorCode::AuthInvalidCredentials,
                translate(locale, MessageKey::InvalidEmailOrPassword).to_string(),
            )
        })?;

    Ok(user)
}

/// 按用户 ID 获取内部用户实体。
pub async fn get_user(state: &AppState, user_id: Uuid) -> Result<Option<User>, AppError> {
    let locale = state.config.base.default_locale;

    user_repo::get_user(&state.db, user_id).await.map_err(|_| {
        AppError::InternalWithMessage(
            translate(locale, MessageKey::InternalServerError).to_string(),
        )
    })
}

/// 执行用户唯一性检查、密码哈希与数据库建档。
async fn create_user_record(
    state: &AppState,
    input: &CreateIdentityUser,
) -> Result<User, AppError> {
    let locale = state.config.base.default_locale;

    let existing = user_repo::get_user_by_email(&state.db, &input.email)
        .await
        .map_err(|_| {
            AppError::InternalWithMessage(
                translate(locale, MessageKey::InternalServerError).to_string(),
            )
        })?;

    if existing.is_some() {
        return Err(AppError::BadRequestWithCode(
            ErrorCode::UserEmailExists,
            translate(locale, MessageKey::EmailAlreadyExists).to_string(),
        ));
    }

    let salt = SaltString::generate(&mut OsRng);
    let password_hash = Argon2::default()
        .hash_password(input.password.as_bytes(), &salt)
        .map_err(|_| {
            AppError::InternalWithMessage(
                translate(locale, MessageKey::InternalServerError).to_string(),
            )
        })?
        .to_string();

    user_repo::create_user(&state.db, &input.name, &input.email, &password_hash)
        .await
        .map_err(|_| {
            AppError::InternalWithMessage(
                translate(locale, MessageKey::InternalServerError).to_string(),
            )
        })
}
