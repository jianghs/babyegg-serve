pub struct RoleKey;

impl RoleKey {
    pub const USER: &'static str = "user";
    pub const ADMIN: &'static str = "admin";
}

pub struct PermissionKey;

impl PermissionKey {
    pub const USERS_READ: &'static str = "users:read";
    pub const USERS_WRITE: &'static str = "users:write";
    pub const WILDCARD: &'static str = "*";
}
