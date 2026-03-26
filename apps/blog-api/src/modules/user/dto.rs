use serde::Deserialize;

use app_foundation::i18n::{translate, MessageKey};
use app_foundation::{
    AppError, ErrorCode, ListQuery, Locale, PageResponse, ValidationDetail, ValidationReason,
};

use crate::modules::identity::model::UserResponse;

/// 创建用户请求。
///
/// 用于 `POST /users`。
/// 当前该接口也要求提供密码，方便直接复用身份域建档与密码哈希逻辑。
#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    /// 用户显示名。
    pub name: String,
    /// 登录邮箱。
    pub email: String,
    /// 明文密码。
    pub password: String,
}

/// 更新用户请求。
///
/// 用于 `PUT /users/{id}`。
/// 当前仅允许更新用户显示名，不涉及邮箱、密码或角色修改。
#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    /// 新的用户显示名。
    pub name: String,
}

impl UpdateUserRequest {
    /// 校验用户更新请求。
    ///
    /// 当前规则：
    /// - `name` 去除空白后不能为空
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
///
/// 复用基础库 `ListQuery`：
/// - `page`：页码，从 1 开始
/// - `page_size`：每页条数
/// - `sort` / `order` / `filter`：当前保留，用户列表接口暂未使用
pub type UserListQuery = ListQuery;

/// 用户列表响应。
///
/// 分页项中的元素类型为 [`UserResponse`]。
pub type UserListResponse = PageResponse<UserResponse>;
