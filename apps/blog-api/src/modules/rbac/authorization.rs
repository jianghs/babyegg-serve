use app_foundation::i18n::{translate, MessageKey};
use app_foundation::{AppError, ErrorCode};

use crate::{modules::rbac::context::AccessContext, state::AppState};

/// 要求当前访问主体具备指定角色，否则返回 403。
pub fn require_role(
    state: &AppState,
    current_user: &AccessContext,
    role: &str,
) -> Result<(), AppError> {
    if current_user.has_role(role) {
        return Ok(());
    }

    Err(AppError::ForbiddenWithCode(
        ErrorCode::AuthForbiddenRole,
        translate(state.config.base.default_locale, MessageKey::ForbiddenRole).to_string(),
    ))
}

/// 要求当前访问主体具备指定权限，否则返回 403。
pub fn require_scope(
    state: &AppState,
    current_user: &AccessContext,
    scope: &str,
) -> Result<(), AppError> {
    if current_user.has_scope(scope) {
        return Ok(());
    }

    Err(AppError::ForbiddenWithCode(
        ErrorCode::AuthForbiddenScope,
        translate(state.config.base.default_locale, MessageKey::ForbiddenScope).to_string(),
    ))
}
