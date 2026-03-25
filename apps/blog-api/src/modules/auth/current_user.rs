use uuid::Uuid;

/// 认证中间件注入的当前用户上下文。
#[derive(Debug, Clone)]
pub struct CurrentUser {
    pub user_id: Uuid,
    pub roles: Vec<String>,
    pub scopes: Vec<String>,
}

impl CurrentUser {
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.iter().any(|r| r == role)
    }

    pub fn has_scope(&self, scope: &str) -> bool {
        self.scopes.iter().any(|s| s == "*" || s == scope)
    }
}
