use serde::Deserialize;

use app_foundation::i18n::{translate, MessageKey};
use app_foundation::{
    AppError, ErrorCode, ListQuery, Locale, PageResponse, ValidationDetail, ValidationReason,
};

use crate::modules::identity::model::UserResponse;

/// 创建用户请求。
/// 当前该接口也要求提供密码，方便与认证链路保持一致。
#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub name: String,
    pub email: String,
    pub password: String,
}

/// 更新用户请求。
#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    pub name: String,
}

impl UpdateUserRequest {
    pub fn validate(&self, locale: Locale) -> Result<(), AppError> {
        if self.name.trim().is_empty() {
            return Err(AppError::BadRequestWithDetails(
                ErrorCode::UserNameEmpty,
                translate(locale, MessageKey::NameCannotBeEmpty).to_string(),
                vec![ValidationDetail::new("name", ValidationReason::Required)],
            ));
        }

        Ok(())
    }
}

/// 分页查询参数。
pub type UserListQuery = ListQuery;

/// 用户列表响应。
pub type UserListResponse = PageResponse<UserResponse>;
