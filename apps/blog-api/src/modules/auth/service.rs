use app_foundation::i18n::{translate, MessageKey};
use app_foundation::{AppError, ErrorCode};
use argon2::password_hash::rand_core::OsRng;
use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

use crate::{
    db::{auth_repo, user_repo},
    modules::{
        auth::{
            dto::{
                LoginRequest, LoginResponse, LogoutRequest, RefreshRequest, RegisterRequest,
                TokenResponse,
            },
            jwt,
        },
        user::model::UserResponse,
    },
    state::AppState,
};

/// 注册用户。
pub async fn register(state: &AppState, req: RegisterRequest) -> Result<UserResponse, AppError> {
    let locale = state.config.base.default_locale;
    req.validate(locale)?;

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
    req.validate(locale)?;

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

    let user_id = user.id;
    let user_response: UserResponse = user.into();
    let token = issue_token_pair(state, user_id).await?;

    Ok(LoginResponse {
        token,
        user: user_response,
    })
}

/// 使用 refresh_token 换取新的 access_token + refresh_token。
pub async fn refresh(state: &AppState, req: RefreshRequest) -> Result<LoginResponse, AppError> {
    let locale = state.config.base.default_locale;
    req.validate(locale)?;

    let user_id = auth_repo::find_valid_refresh_token_user_id(&state.db, &req.refresh_token)
        .await
        .map_err(|_| {
            AppError::InternalWithMessage(
                translate(locale, MessageKey::InternalServerError).to_string(),
            )
        })?
        .ok_or(AppError::BadRequestWithCode(
            ErrorCode::AuthInvalidRefreshToken,
            translate(locale, MessageKey::InvalidRefreshToken).to_string(),
        ))?;

    let revoked = auth_repo::revoke_refresh_token(&state.db, &req.refresh_token)
        .await
        .map_err(|_| {
            AppError::InternalWithMessage(
                translate(locale, MessageKey::InternalServerError).to_string(),
            )
        })?;
    if !revoked {
        return Err(AppError::BadRequestWithCode(
            ErrorCode::AuthInvalidRefreshToken,
            translate(locale, MessageKey::InvalidRefreshToken).to_string(),
        ));
    }

    let user = user_repo::get_user(&state.db, user_id)
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

    let token = issue_token_pair(state, user_id).await?;

    Ok(LoginResponse {
        token,
        user: user.into(),
    })
}

/// 撤销 refresh_token。
pub async fn logout(state: &AppState, req: LogoutRequest) -> Result<(), AppError> {
    let locale = state.config.base.default_locale;
    req.validate(locale)?;

    let revoked = auth_repo::revoke_refresh_token(&state.db, &req.refresh_token)
        .await
        .map_err(|_| {
            AppError::InternalWithMessage(
                translate(locale, MessageKey::InternalServerError).to_string(),
            )
        })?;

    if !revoked {
        return Err(AppError::BadRequestWithCode(
            ErrorCode::AuthInvalidRefreshToken,
            translate(locale, MessageKey::InvalidRefreshToken).to_string(),
        ));
    }

    Ok(())
}

async fn issue_token_pair(
    state: &AppState,
    user_id: uuid::Uuid,
) -> Result<TokenResponse, AppError> {
    let locale = state.config.base.default_locale;
    let access_token = jwt::create_token(
        user_id,
        &state.config.jwt_secret,
        state.config.jwt_expire_seconds,
    )
    .map_err(|_| {
        AppError::InternalWithMessage(
            translate(locale, MessageKey::InternalServerError).to_string(),
        )
    })?;

    let refresh_token = uuid::Uuid::new_v4().to_string();
    let refresh_expires_at = time::OffsetDateTime::now_utc()
        + time::Duration::seconds(state.config.jwt_refresh_expire_seconds);

    auth_repo::create_refresh_token(&state.db, user_id, &refresh_token, refresh_expires_at)
        .await
        .map_err(|_| {
            AppError::InternalWithMessage(
                translate(locale, MessageKey::InternalServerError).to_string(),
            )
        })?;

    Ok(TokenResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: state.config.jwt_expire_seconds,
        refresh_expires_in: state.config.jwt_refresh_expire_seconds,
    })
}
