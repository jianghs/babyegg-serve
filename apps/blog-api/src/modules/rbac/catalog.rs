use crate::modules::rbac::keys::{PermissionKey, RoleKey};

pub const ROLE_KEYS: &[&str] = &[RoleKey::ADMIN, RoleKey::USER];

pub const PERMISSION_KEYS: &[&str] = &[
    PermissionKey::WILDCARD,
    PermissionKey::USERS_READ,
    PermissionKey::USERS_WRITE,
];

pub const ROLE_PERMISSION_PAIRS: &[(&str, &str)] = &[
    (RoleKey::ADMIN, PermissionKey::WILDCARD),
    (RoleKey::USER, PermissionKey::USERS_READ),
];
