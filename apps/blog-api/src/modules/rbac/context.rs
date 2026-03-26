use uuid::Uuid;

use crate::modules::{auth::jwt::Claims, rbac::keys::PermissionKey};

/// 认证成功后注入请求上下文的授权主体。
#[derive(Debug, Clone)]
pub struct AccessContext {
    /// 当前访问用户 ID。
    pub user_id: Uuid,
    /// 当前用户拥有的角色集合。
    pub roles: Vec<String>,
    /// 当前用户拥有的权限范围集合。
    pub scopes: Vec<String>,
}

impl AccessContext {
    /// 判断当前用户是否拥有指定角色。
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.iter().any(|r| r == role)
    }

    /// 判断当前用户是否拥有指定权限，支持 `*` 通配权限。
    pub fn has_scope(&self, scope: &str) -> bool {
        self.scopes
            .iter()
            .any(|s| s == PermissionKey::WILDCARD || s == scope)
    }
}

impl TryFrom<Claims> for AccessContext {
    type Error = uuid::Error;

    /// 将 JWT 声明体转换为运行时访问上下文。
    fn try_from(value: Claims) -> Result<Self, Self::Error> {
        Ok(Self {
            user_id: value.sub.parse::<Uuid>()?,
            roles: value.roles,
            scopes: value.scopes,
        })
    }
}
