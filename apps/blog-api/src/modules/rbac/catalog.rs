use crate::modules::rbac::keys::{PermissionKey, RoleKey};

/// 系统允许持久化的全部角色键。
pub const ROLE_KEYS: &[&str] = &[RoleKey::ADMIN, RoleKey::USER];

/// 系统允许持久化的全部权限键。
pub const PERMISSION_KEYS: &[&str] = &[
    PermissionKey::WILDCARD,
    PermissionKey::USERS_READ,
    PermissionKey::USERS_WRITE,
];

/// 角色与默认权限的映射关系。
pub const ROLE_PERMISSION_PAIRS: &[(&str, &str)] = &[
    (RoleKey::ADMIN, PermissionKey::WILDCARD),
    (RoleKey::USER, PermissionKey::USERS_READ),
];
