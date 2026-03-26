use sqlx::PgPool;

use crate::config::AppConfig;

/// 业务服务状态。
/// 这里组合数据库池、HTTP 客户端和业务配置。
#[derive(Clone)]
pub struct AppState {
    /// PostgreSQL 连接池。
    pub db: PgPool,
    /// 对外 HTTP 客户端。
    pub http_client: reqwest::Client,
    /// 业务服务配置。
    pub config: AppConfig,
}

impl AppState {
    /// 构造业务服务运行时状态。
    pub fn new(db: PgPool, config: AppConfig) -> Self {
        Self {
            db,
            http_client: reqwest::Client::new(),
            config,
        }
    }
}
