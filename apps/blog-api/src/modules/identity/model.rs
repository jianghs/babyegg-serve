use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::OffsetDateTime;
use uuid::Uuid;

/// 数据库中的用户实体。
/// 注意：该结构包含 password_hash，只用于服务内部，不直接对外返回。
#[derive(Debug, Clone, Deserialize, FromRow)]
pub struct User {
    /// 用户主键。
    pub id: Uuid,
    /// 用户显示名。
    pub name: String,
    /// 登录邮箱。
    pub email: String,
    /// Argon2 哈希后的密码，仅供服务内部使用。
    pub password_hash: String,
    /// 创建时间。
    pub created_at: OffsetDateTime,
    /// 更新时间。
    pub updated_at: OffsetDateTime,
}

/// 对外返回的用户响应。
#[derive(Debug, Clone, Serialize)]
pub struct UserResponse {
    /// 用户主键字符串。
    pub id: String,
    /// 用户显示名。
    pub name: String,
    /// 登录邮箱。
    pub email: String,
    /// RFC3339 格式的创建时间。
    pub created_at: String,
    /// RFC3339 格式的更新时间。
    pub updated_at: String,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id.to_string(),
            name: user.name,
            email: user.email,
            created_at: format_datetime(user.created_at),
            updated_at: format_datetime(user.updated_at),
        }
    }
}

fn format_datetime(dt: OffsetDateTime) -> String {
    dt.format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_else(|_| dt.unix_timestamp().to_string())
}
