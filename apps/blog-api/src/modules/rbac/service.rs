use app_foundation::i18n::{translate, MessageKey};
use app_foundation::AppError;
use uuid::Uuid;

use crate::{db::rbac_repo, modules::rbac::keys::RoleKey, state::AppState};

pub async fn assign_role_or_fail(
    state: &AppState,
    user_id: Uuid,
    role_key: &str,
) -> Result<(), AppError> {
    let locale = state.config.base.default_locale;

    let assigned = rbac_repo::assign_role_by_key(&state.db, user_id, role_key)
        .await
        .map_err(|_| {
            AppError::InternalWithMessage(
                translate(locale, MessageKey::InternalServerError).to_string(),
            )
        })?;

    if !assigned {
        return Err(AppError::InternalWithMessage(
            translate(locale, MessageKey::InternalServerError).to_string(),
        ));
    }

    Ok(())
}

pub async fn build_claims(
    state: &AppState,
    user_id: Uuid,
) -> Result<(Vec<String>, Vec<String>), AppError> {
    let locale = state.config.base.default_locale;
    let (mut roles, mut scopes) = rbac_repo::get_user_roles_and_scopes(&state.db, user_id)
        .await
        .map_err(|_| {
            AppError::InternalWithMessage(
                translate(locale, MessageKey::InternalServerError).to_string(),
            )
        })?;

    if roles.is_empty() {
        assign_role_or_fail(state, user_id, RoleKey::USER).await?;
        (roles, scopes) = rbac_repo::get_user_roles_and_scopes(&state.db, user_id)
            .await
            .map_err(|_| {
                AppError::InternalWithMessage(
                    translate(locale, MessageKey::InternalServerError).to_string(),
                )
            })?;
    }

    Ok((roles, scopes))
}
