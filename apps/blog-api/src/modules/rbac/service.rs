use app_foundation::i18n::{translate, MessageKey};
use app_foundation::{AppError, ErrorCode};
use uuid::Uuid;

use crate::{
    db::{rbac_repo, user_repo},
    modules::rbac::{
        dto::{
            AssignUserRoleRequest, PermissionResponse, RoleResponse, UserAccessResponse,
            UserRoleResponse,
        },
        keys::{Permission, Role, RoleKey},
    },
    state::AppState,
};

/// 为用户分配指定角色；若角色键不存在则视为内部错误。
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

/// 构建 JWT 所需的角色与权限声明。
///
/// 当用户尚未分配任何角色时，会自动补一个默认 `user` 角色。
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

/// 查询全部角色及其权限矩阵。
pub async fn list_roles(state: &AppState) -> Result<Vec<RoleResponse>, AppError> {
    let locale = state.config.base.default_locale;
    let roles = rbac_repo::list_roles(&state.db).await.map_err(|_| {
        AppError::InternalWithMessage(
            translate(locale, MessageKey::InternalServerError).to_string(),
        )
    })?;

    let mut responses = Vec::with_capacity(roles.len());
    for role in roles {
        let permissions = rbac_repo::list_permissions_by_role_key(&state.db, &role.role_key)
            .await
            .map_err(|_| {
                AppError::InternalWithMessage(
                    translate(locale, MessageKey::InternalServerError).to_string(),
                )
            })?;
        responses.push(RoleResponse {
            role_key: Role::from_key(&role.role_key).ok_or(AppError::InternalWithMessage(
                translate(locale, MessageKey::InternalServerError).to_string(),
            ))?,
            description: role.description,
            permissions: permissions
                .into_iter()
                .map(|permission| {
                    Permission::from_key(&permission).ok_or(AppError::InternalWithMessage(
                        translate(locale, MessageKey::InternalServerError).to_string(),
                    ))
                })
                .collect::<Result<Vec<_>, _>>()?,
        });
    }

    Ok(responses)
}

/// 查询全部权限定义。
pub async fn list_permissions(state: &AppState) -> Result<Vec<PermissionResponse>, AppError> {
    let locale = state.config.base.default_locale;
    let permissions = rbac_repo::list_permissions(&state.db).await.map_err(|_| {
        AppError::InternalWithMessage(
            translate(locale, MessageKey::InternalServerError).to_string(),
        )
    })?;

    permissions
        .into_iter()
        .map(|permission| {
            Ok(PermissionResponse {
                permission_key: Permission::from_key(&permission.permission_key).ok_or(
                    AppError::InternalWithMessage(
                        translate(locale, MessageKey::InternalServerError).to_string(),
                    ),
                )?,
                description: permission.description,
            })
        })
        .collect::<Result<Vec<_>, _>>()
}

/// 查询某个用户当前的角色与权限。
pub async fn get_user_access(
    state: &AppState,
    user_id: Uuid,
) -> Result<UserAccessResponse, AppError> {
    let locale = state.config.base.default_locale;
    let user_exists = user_repo::get_user(&state.db, user_id).await.map_err(|_| {
        AppError::InternalWithMessage(
            translate(locale, MessageKey::InternalServerError).to_string(),
        )
    })?;

    if user_exists.is_none() {
        return Err(AppError::NotFoundWithCode(
            ErrorCode::NotFound,
            translate(locale, MessageKey::NotFound).to_string(),
        ));
    }

    let roles = rbac_repo::list_user_roles(&state.db, user_id)
        .await
        .map_err(|_| {
            AppError::InternalWithMessage(
                translate(locale, MessageKey::InternalServerError).to_string(),
            )
        })?;
    let (_role_keys, scopes) = build_claims(state, user_id).await?;

    Ok(UserAccessResponse {
        user_id: user_id.to_string(),
        roles: roles
            .into_iter()
            .map(|role| {
                Ok(UserRoleResponse {
                    role_key: Role::from_key(&role.role_key).ok_or(
                        AppError::InternalWithMessage(
                            translate(locale, MessageKey::InternalServerError).to_string(),
                        ),
                    )?,
                    description: role.description,
                })
            })
            .collect::<Result<Vec<_>, _>>()?,
        scopes: scopes
            .into_iter()
            .map(|scope| {
                Permission::from_key(&scope).ok_or(AppError::InternalWithMessage(
                    translate(locale, MessageKey::InternalServerError).to_string(),
                ))
            })
            .collect::<Result<Vec<_>, _>>()?,
    })
}

/// 给指定用户分配角色，并返回更新后的角色与权限。
pub async fn assign_user_role(
    state: &AppState,
    user_id: Uuid,
    req: AssignUserRoleRequest,
) -> Result<UserAccessResponse, AppError> {
    let locale = state.config.base.default_locale;
    req.validate(locale)?;

    let user_exists = user_repo::get_user(&state.db, user_id).await.map_err(|_| {
        AppError::InternalWithMessage(
            translate(locale, MessageKey::InternalServerError).to_string(),
        )
    })?;
    if user_exists.is_none() {
        return Err(AppError::NotFoundWithCode(
            ErrorCode::NotFound,
            translate(locale, MessageKey::NotFound).to_string(),
        ));
    }

    let assigned = rbac_repo::assign_role_by_key(&state.db, user_id, req.role_key.trim())
        .await
        .map_err(|_| {
            AppError::InternalWithMessage(
                translate(locale, MessageKey::InternalServerError).to_string(),
            )
        })?;

    if !assigned {
        return Err(AppError::BadRequestWithCode(
            ErrorCode::InvalidParam,
            "invalid role_key".to_string(),
        ));
    }

    get_user_access(state, user_id).await
}
