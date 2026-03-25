use sqlx::PgPool;

use crate::config::AppConfig;

/// 业务服务状态。
/// 这里组合数据库池、HTTP 客户端和业务配置。
#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub http_client: reqwest::Client,
    pub config: AppConfig,
}

impl AppState {
    pub fn new(db: PgPool, config: AppConfig) -> Self {
        Self {
            db,
            http_client: reqwest::Client::new(),
            config,
        }
    }
}
