use anyhow::{Context, Result};

use app_foundation::BaseConfig;

/// 业务服务自己的配置。
#[derive(Debug, Clone)]
pub struct AppConfig {
    /// 基础配置，来自基础库
    pub base: BaseConfig,
    /// 数据库连接串
    pub database_url: String,
    /// 外部 HTTP 示例服务地址
    pub httpbin_base_url: String,
    /// JWT 密钥
    pub jwt_secret: String,
    /// JWT 有效期，单位秒
    pub jwt_expire_seconds: i64,
    /// Refresh Token 有效期，单位秒
    pub jwt_refresh_expire_seconds: i64,
}

impl AppConfig {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            base: BaseConfig::from_env()?,
            database_url: std::env::var("DATABASE_URL").context("DATABASE_URL is required")?,
            httpbin_base_url: std::env::var("HTTPBIN_BASE_URL")
                .unwrap_or_else(|_| "https://httpbin.org".to_string()),
            jwt_secret: std::env::var("JWT_SECRET")
                .unwrap_or_else(|_| "change_me_in_production".to_string()),
            jwt_expire_seconds: std::env::var("JWT_EXPIRE_SECONDS")
                .unwrap_or_else(|_| "86400".to_string())
                .parse::<i64>()
                .context("JWT_EXPIRE_SECONDS must be a valid i64")?,
            jwt_refresh_expire_seconds: std::env::var("JWT_REFRESH_EXPIRE_SECONDS")
                .unwrap_or_else(|_| "604800".to_string())
                .parse::<i64>()
                .context("JWT_REFRESH_EXPIRE_SECONDS must be a valid i64")?,
        })
    }
}
