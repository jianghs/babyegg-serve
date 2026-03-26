use uuid::Uuid;

use crate::modules::{auth::jwt::Claims, rbac::keys::PermissionKey};

/// 认证成功后注入请求上下文的授权主体。
#[derive(Debug, Clone)]
pub struct AccessContext {
    pub user_id: Uuid,
    pub roles: Vec<String>,
    pub scopes: Vec<String>,
}

impl AccessContext {
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.iter().any(|r| r == role)
    }

    pub fn has_scope(&self, scope: &str) -> bool {
        self.scopes
            .iter()
            .any(|s| s == PermissionKey::WILDCARD || s == scope)
    }
}

impl TryFrom<Claims> for AccessContext {
    type Error = uuid::Error;

    fn try_from(value: Claims) -> Result<Self, Self::Error> {
        Ok(Self {
            user_id: value.sub.parse::<Uuid>()?,
            roles: value.roles,
            scopes: value.scopes,
        })
    }
}
