use serde::Deserialize;

use app_foundation::i18n::{translate, MessageKey};
use app_foundation::{
    AppError, ErrorCode, Locale, PageResponse, SortOrder, ValidationDetail, ValidationReason,
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

/// 用户列表可选排序字段。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserSortField {
    /// 按创建时间排序。
    CreatedAt,
    /// 按更新时间排序。
    UpdatedAt,
    /// 按用户名排序。
    Name,
    /// 按邮箱排序。
    Email,
}

impl UserSortField {
    /// 将查询参数中的排序字段解析为受限枚举，避免自由拼接 SQL。
    pub fn parse(value: Option<&str>) -> Result<Self, &'static str> {
        match value.map(str::trim).filter(|item| !item.is_empty()) {
            None => Ok(Self::CreatedAt),
            Some("created_at") => Ok(Self::CreatedAt),
            Some("updated_at") => Ok(Self::UpdatedAt),
            Some("name") => Ok(Self::Name),
            Some("email") => Ok(Self::Email),
            Some(_) => Err("sort"),
        }
    }
}

/// 用户列表归一化查询参数。
#[derive(Debug, Clone)]
pub struct NormalizedUserListQuery {
    /// 页码，从 1 开始。
    pub page: i64,
    /// 每页条数。
    pub page_size: i64,
    /// 排序字段。
    pub sort: UserSortField,
    /// 排序方向。
    pub order: SortOrder,
    /// 过滤关键词。
    pub filter: Option<String>,
}

/// 分页查询参数。
///
/// 支持：
/// - `page` / `page_size`：分页
/// - `sort`：`created_at` / `updated_at` / `name` / `email`
/// - `order`：`asc` / `desc`
/// - `filter`：按 `name` / `email` 模糊匹配
#[derive(Debug, Clone, Deserialize)]
pub struct UserListQuery {
    /// 页码，从 1 开始；为空时使用默认值。
    pub page: Option<i64>,
    /// 每页条数；为空时使用默认值。
    pub page_size: Option<i64>,
    /// 排序字段。
    pub sort: Option<String>,
    /// 排序方向。
    pub order: Option<SortOrder>,
    /// 按用户名或邮箱做模糊搜索。
    pub filter: Option<String>,
}

impl UserListQuery {
    /// 将原始查询参数校验并归一化为业务可执行的查询结构。
    pub fn normalize(
        self,
        locale: Locale,
        default_page: i64,
        default_page_size: i64,
        max_page_size: i64,
    ) -> Result<NormalizedUserListQuery, AppError> {
        let sort = UserSortField::parse(self.sort.as_deref()).map_err(|field| {
            AppError::BadRequestWithDetails(
                ErrorCode::InvalidParam,
                if matches!(locale, Locale::ZhCn) {
                    "sort 参数不合法".to_string()
                } else {
                    "invalid sort parameter".to_string()
                },
                vec![ValidationDetail::new(
                    field,
                    ValidationReason::InvalidFormat,
                )],
            )
        })?;

        Ok(NormalizedUserListQuery {
            page: self.page.unwrap_or(default_page).max(1),
            page_size: self
                .page_size
                .unwrap_or(default_page_size)
                .clamp(1, max_page_size),
            sort,
            order: self.order.unwrap_or(SortOrder::Desc),
            filter: self
                .filter
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty()),
        })
    }
}

/// 用户列表响应。
///
/// 分页项中的元素类型为 [`UserResponse`]。
pub type UserListResponse = PageResponse<UserResponse>;
