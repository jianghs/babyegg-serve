use app_foundation::i18n::{translate, MessageKey};
use app_foundation::{AppError, ErrorCode};

use crate::modules::auth::current_user::CurrentUser;
use crate::state::AppState;

pub fn require_role(
    state: &AppState,
    current_user: &CurrentUser,
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

pub fn require_scope(
    state: &AppState,
    current_user: &CurrentUser,
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
