use app_foundation::i18n::{translate, MessageKey};
use app_foundation::{AppError, ErrorCode};
use argon2::password_hash::rand_core::OsRng;
use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

use crate::{
    db::user_repo,
    modules::{
        auth::{
            dto::{LoginRequest, LoginResponse, RegisterRequest, TokenResponse},
            jwt,
        },
        user::model::UserResponse,
    },
    state::AppState,
};

/// 注册用户。
pub async fn register(state: &AppState, req: RegisterRequest) -> Result<UserResponse, AppError> {
    let locale = state.config.base.default_locale;

    if req.name.trim().is_empty() {
        return Err(AppError::BadRequestWithCode(
            ErrorCode::UserNameEmpty,
            translate(locale, MessageKey::NameCannotBeEmpty).to_string(),
        ));
    }

    if req.email.trim().is_empty() {
        return Err(AppError::BadRequestWithCode(
            ErrorCode::UserEmailEmpty,
            translate(locale, MessageKey::EmailCannotBeEmpty).to_string(),
        ));
    }

    if req.password.len() < 6 {
        return Err(AppError::BadRequestWithCode(
            ErrorCode::UserPasswordTooShort,
            translate(locale, MessageKey::PasswordTooShort).to_string(),
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
        return Err(AppError::BadRequestWithCode(
            ErrorCode::UserEmailExists,
            translate(locale, MessageKey::EmailAlreadyExists).to_string(),
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

    Ok(user.into())
}

/// 登录。
pub async fn login(state: &AppState, req: LoginRequest) -> Result<LoginResponse, AppError> {
    let locale = state.config.base.default_locale;

    if req.email.trim().is_empty() {
        return Err(AppError::BadRequestWithCode(
            ErrorCode::UserEmailEmpty,
            translate(locale, MessageKey::EmailCannotBeEmpty).to_string(),
        ));
    }

    if req.password.is_empty() {
        return Err(AppError::BadRequestWithCode(
            ErrorCode::UserPasswordEmpty,
            translate(locale, MessageKey::PasswordCannotBeEmpty).to_string(),
        ));
    }

    let user = user_repo::get_user_by_email(&state.db, &req.email)
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
        .verify_password(req.password.as_bytes(), &parsed_hash)
        .map_err(|_| {
            AppError::BadRequestWithCode(
                ErrorCode::AuthInvalidCredentials,
                translate(locale, MessageKey::InvalidEmailOrPassword).to_string(),
            )
        })?;

    let token = jwt::create_token(
        user.id,
        &state.config.jwt_secret,
        state.config.jwt_expire_seconds,
    )
    .map_err(|_| {
        AppError::InternalWithMessage(
            translate(locale, MessageKey::InternalServerError).to_string(),
        )
    })?;

    let user_response: UserResponse = user.into();

    Ok(LoginResponse {
        token: TokenResponse {
            access_token: token,
            token_type: "Bearer".to_string(),
            expires_in: state.config.jwt_expire_seconds,
        },
        user: user_response,
    })
}
