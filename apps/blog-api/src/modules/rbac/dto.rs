use app_foundation::{AppError, ErrorCode, Locale, ValidationDetail, ValidationReason};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct RoleResponse {
    pub role_key: String,
    pub description: String,
    pub permissions: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct PermissionResponse {
    pub permission_key: String,
    pub description: String,
}

#[derive(Debug, Serialize)]
pub struct UserRoleResponse {
    pub role_key: String,
    pub description: String,
}

#[derive(Debug, Serialize)]
pub struct UserAccessResponse {
    pub user_id: String,
    pub roles: Vec<UserRoleResponse>,
    pub scopes: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct AssignUserRoleRequest {
    pub role_key: String,
}

impl AssignUserRoleRequest {
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
