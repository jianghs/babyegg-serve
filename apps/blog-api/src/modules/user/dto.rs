use serde::Deserialize;

use app_foundation::PageResponse;

use crate::modules::user::model::UserResponse;

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

/// 分页查询参数。
#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

/// 用户列表响应。
pub type UserListResponse = PageResponse<UserResponse>;
