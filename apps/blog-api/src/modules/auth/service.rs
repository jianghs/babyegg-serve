//! 认证会话服务负责登录、刷新与登出流程编排。

use app_foundation::i18n::{translate, MessageKey};
use app_foundation::{AppError, ErrorCode};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::{
    db::session_repo,
    modules::{
        auth::{
            dto::{
                LoginRequest, LoginResponse, LogoutRequest, RefreshRequest,
                RevokeAllSessionsResponse, SessionResponse, TokenResponse,
            },
            jwt,
        },
        identity, rbac,
    },
    state::AppState,
};

/// 校验用户凭证并签发新的访问令牌与刷新令牌。
///
/// 处理流程：
/// - 校验登录请求的基础字段
/// - 通过身份域校验邮箱与密码
/// - 生成 access token
/// - 创建并持久化 refresh token
pub async fn login(state: &AppState, req: LoginRequest) -> Result<LoginResponse, AppError> {
    let locale = state.config.base.default_locale;
    req.validate(locale)?;

    let user = identity::service::verify_credentials(state, &req.email, &req.password).await?;
    let user_id = user.id;
    let token = issue_token_pair(state, user_id).await?;

    Ok(LoginResponse {
        token,
        user: user.into(),
    })
}

/// 使用 refresh token 轮换出一组新的访问令牌与刷新令牌。
///
/// 当前实现采用 refresh token 轮换策略：
/// - 旧 refresh token 必须有效
/// - 旧 refresh token 会先被撤销
/// - 成功后返回一组新的 access token 与 refresh token
///
/// 若 refresh token 无效、已撤销或已过期，返回
/// [`ErrorCode::AuthInvalidRefreshToken`]。
pub async fn refresh(state: &AppState, req: RefreshRequest) -> Result<LoginResponse, AppError> {
    let locale = state.config.base.default_locale;
    req.validate(locale)?;

    let user_id = session_repo::find_valid_refresh_token_user_id(&state.db, &req.refresh_token)
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

    let revoked = session_repo::revoke_refresh_token(&state.db, &req.refresh_token)
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

    let user =
        identity::service::get_user(state, user_id)
            .await?
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

/// 撤销指定 refresh token，使其后续不能再用于续签。
///
/// 若传入 token 不存在、已被撤销或不再有效，返回
/// [`ErrorCode::AuthInvalidRefreshToken`]。
pub async fn logout(state: &AppState, req: LogoutRequest) -> Result<(), AppError> {
    let locale = state.config.base.default_locale;
    req.validate(locale)?;

    let revoked = session_repo::revoke_refresh_token(&state.db, &req.refresh_token)
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

/// 生成访问令牌与刷新令牌，并持久化刷新会话。
///
/// 该函数内部会根据 RBAC 当前状态动态构建 JWT 中的角色与权限声明。
async fn issue_token_pair(state: &AppState, user_id: Uuid) -> Result<TokenResponse, AppError> {
    let locale = state.config.base.default_locale;
    let (roles, scopes) = rbac::service::build_claims(state, user_id).await?;

    let access_token = jwt::create_token(
        user_id,
        &state.config.auth.jwt_secret,
        state.config.auth.access_token_ttl_seconds,
        roles,
        scopes,
    )
    .map_err(|_| {
        AppError::InternalWithMessage(
            translate(locale, MessageKey::InternalServerError).to_string(),
        )
    })?;

    let refresh_token = uuid::Uuid::new_v4().to_string();
    let refresh_expires_at = time::OffsetDateTime::now_utc()
        + time::Duration::seconds(state.config.auth.refresh_token_ttl_seconds);

    let session_id =
        session_repo::create_refresh_token(&state.db, user_id, &refresh_token, refresh_expires_at)
            .await
            .map_err(|_| {
                AppError::InternalWithMessage(
                    translate(locale, MessageKey::InternalServerError).to_string(),
                )
            })?;

    Ok(TokenResponse {
        session_id,
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: state.config.auth.access_token_ttl_seconds,
        refresh_expires_in: state.config.auth.refresh_token_ttl_seconds,
    })
}

/// 列出当前用户的全部会话。
pub async fn list_sessions(
    state: &AppState,
    user_id: Uuid,
) -> Result<Vec<SessionResponse>, AppError> {
    let locale = state.config.base.default_locale;
    let sessions = session_repo::list_sessions_by_user_id(&state.db, user_id)
        .await
        .map_err(|_| {
            AppError::InternalWithMessage(
                translate(locale, MessageKey::InternalServerError).to_string(),
            )
        })?;

    Ok(sessions
        .into_iter()
        .map(|session| {
            let now = OffsetDateTime::now_utc();
            SessionResponse {
                id: session.id,
                user_id: session.user_id,
                created_at: session
                    .created_at
                    .format(&time::format_description::well_known::Rfc3339)
                    .expect("invalid created_at"),
                expires_at: session
                    .expires_at
                    .format(&time::format_description::well_known::Rfc3339)
                    .expect("invalid expires_at"),
                revoked_at: session.revoked_at.map(|value| {
                    value
                        .format(&time::format_description::well_known::Rfc3339)
                        .expect("invalid revoked_at")
                }),
                is_active: session.revoked_at.is_none() && session.expires_at > now,
            }
        })
        .collect())
}

/// 撤销当前用户的指定会话。
pub async fn revoke_session(
    state: &AppState,
    user_id: Uuid,
    session_id: Uuid,
) -> Result<(), AppError> {
    let locale = state.config.base.default_locale;
    let revoked = session_repo::revoke_session_by_id(&state.db, user_id, session_id)
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

/// 撤销当前用户的全部有效会话。
pub async fn revoke_all_sessions(
    state: &AppState,
    user_id: Uuid,
) -> Result<RevokeAllSessionsResponse, AppError> {
    let locale = state.config.base.default_locale;
    let revoked_sessions = session_repo::revoke_all_sessions_by_user_id(&state.db, user_id)
        .await
        .map_err(|_| {
            AppError::InternalWithMessage(
                translate(locale, MessageKey::InternalServerError).to_string(),
            )
        })?;

    Ok(RevokeAllSessionsResponse { revoked_sessions })
}
