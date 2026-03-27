use app_foundation::{AppError, ErrorCode, Locale, ValidationDetail, ValidationReason};
use serde::{Deserialize, Serialize};

use crate::modules::rbac::keys::{Permission, Role};

/// 角色定义响应。
#[derive(Debug, Serialize)]
pub struct RoleResponse {
    /// 角色键。
    pub role_key: Role,
    /// 角色说明。
    pub description: String,
    /// 角色默认拥有的权限集合。
    pub permissions: Vec<Permission>,
}

/// 权限定义响应。
#[derive(Debug, Serialize)]
pub struct PermissionResponse {
    /// 权限键。
    pub permission_key: Permission,
    /// 权限说明。
    pub description: String,
}

/// 用户显式绑定的角色响应。
#[derive(Debug, Serialize)]
pub struct UserRoleResponse {
    /// 角色键。
    pub role_key: Role,
    /// 角色说明。
    pub description: String,
}

/// 用户访问上下文响应。
#[derive(Debug, Serialize)]
pub struct UserAccessResponse {
    /// 用户 ID。
    pub user_id: String,
    /// 用户显式绑定的角色集合。
    pub roles: Vec<UserRoleResponse>,
    /// 当前用户聚合得到的权限集合。
    pub scopes: Vec<Permission>,
}

/// 分配用户角色请求。
#[derive(Debug, Deserialize)]
pub struct AssignUserRoleRequest {
    /// 待分配的角色键。
    pub role_key: String,
}

impl AssignUserRoleRequest {
    /// 校验角色分配请求。
    pub fn validate(&self, locale: Locale) -> Result<(), AppError> {
        if self.role_key.trim().is_empty() {
            return Err(AppError::BadRequestWithDetails(
                ErrorCode::InvalidParam,
                if matches!(locale, Locale::ZhCn) {
                    "role_key 不能为空".to_string()
                } else {
                    "role_key cannot be empty".to_string()
                },
                vec![ValidationDetail::new(
                    "role_key",
                    ValidationReason::Required,
                )],
            ));
        }

        Ok(())
    }
}
