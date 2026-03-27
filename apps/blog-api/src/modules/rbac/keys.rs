use serde::Serialize;

/// 角色键名常量集合。
pub struct RoleKey;

impl RoleKey {
    /// 普通用户角色。
    pub const USER: &'static str = "user";
    /// 管理员角色。
    pub const ADMIN: &'static str = "admin";
}

/// 对外暴露的角色键枚举。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Role {
    /// 普通用户角色。
    #[serde(rename = "user")]
    User,
    /// 管理员角色。
    #[serde(rename = "admin")]
    Admin,
}

impl Role {
    /// 按稳定字符串键解析角色枚举。
    pub fn from_key(value: &str) -> Option<Self> {
        match value {
            RoleKey::USER => Some(Self::User),
            RoleKey::ADMIN => Some(Self::Admin),
            _ => None,
        }
    }
}

/// 权限键名常量集合。
pub struct PermissionKey;

impl PermissionKey {
    /// 读取用户资源权限。
    pub const USERS_READ: &'static str = "users:read";
    /// 写入用户资源权限。
    pub const USERS_WRITE: &'static str = "users:write";
    /// 通配权限，表示拥有全部权限。
    pub const WILDCARD: &'static str = "*";
}

/// 对外暴露的权限键枚举。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Permission {
    /// 读取用户资源权限。
    #[serde(rename = "users:read")]
    UsersRead,
    /// 写入用户资源权限。
    #[serde(rename = "users:write")]
    UsersWrite,
    /// 通配权限，表示拥有全部权限。
    #[serde(rename = "*")]
    Wildcard,
}

impl Permission {
    /// 按稳定字符串键解析权限枚举。
    pub fn from_key(value: &str) -> Option<Self> {
        match value {
            PermissionKey::USERS_READ => Some(Self::UsersRead),
            PermissionKey::USERS_WRITE => Some(Self::UsersWrite),
            PermissionKey::WILDCARD => Some(Self::Wildcard),
            _ => None,
        }
    }
}
