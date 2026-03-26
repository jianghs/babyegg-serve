/// 角色键名常量集合。
pub struct RoleKey;

impl RoleKey {
    /// 普通用户角色。
    pub const USER: &'static str = "user";
    /// 管理员角色。
    pub const ADMIN: &'static str = "admin";
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
