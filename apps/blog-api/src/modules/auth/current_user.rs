use uuid::Uuid;

/// 认证中间件注入的当前用户上下文。
#[derive(Debug, Clone, Copy)]
pub struct CurrentUser {
    pub user_id: Uuid,
}
