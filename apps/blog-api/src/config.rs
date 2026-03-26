use anyhow::{Context, Result};

use app_foundation::BaseConfig;

/// 认证相关配置。
///
/// 这些值会同时影响 access token 签发、refresh token 生命周期，
/// 以及认证中间件对令牌的解析行为。
#[derive(Debug, Clone)]
pub struct AuthConfig {
    /// JWT 签名密钥。
    pub jwt_secret: String,
    /// access token 过期秒数。
    pub access_token_ttl_seconds: i64,
    /// refresh token 过期秒数。
    pub refresh_token_ttl_seconds: i64,
}

/// 业务服务自己的配置。
///
/// 该结构汇总运行 `blog-api` 所需的全部环境配置：
/// - 基础监听配置
/// - 数据库连接配置
/// - 外部 HTTP 服务地址
/// - 认证配置
#[derive(Debug, Clone)]
pub struct AppConfig {
    /// 基础配置，来自基础库
    pub base: BaseConfig,
    /// 数据库连接串
    pub database_url: String,
    /// 外部 HTTP 示例服务地址
    pub httpbin_base_url: String,
    /// 认证相关配置
    pub auth: AuthConfig,
}

impl AppConfig {
    /// 从环境变量加载业务服务配置。
    ///
    /// 当前会读取：
    /// - `DATABASE_URL`
    /// - `HTTPBIN_BASE_URL`
    /// - `JWT_SECRET`
    /// - `JWT_EXPIRE_SECONDS`
    /// - `JWT_REFRESH_EXPIRE_SECONDS`
    ///
    /// 其中 `DATABASE_URL` 为必填，其余字段提供了适合本地开发的默认值。
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            base: BaseConfig::from_env()?,
            database_url: std::env::var("DATABASE_URL").context("DATABASE_URL is required")?,
            httpbin_base_url: std::env::var("HTTPBIN_BASE_URL")
                .unwrap_or_else(|_| "https://httpbin.org".to_string()),
            auth: AuthConfig {
                jwt_secret: std::env::var("JWT_SECRET")
                    .unwrap_or_else(|_| "change_me_in_production".to_string()),
                access_token_ttl_seconds: std::env::var("JWT_EXPIRE_SECONDS")
                    .unwrap_or_else(|_| "86400".to_string())
                    .parse::<i64>()
                    .context("JWT_EXPIRE_SECONDS must be a valid i64")?,
                refresh_token_ttl_seconds: std::env::var("JWT_REFRESH_EXPIRE_SECONDS")
                    .unwrap_or_else(|_| "604800".to_string())
                    .parse::<i64>()
                    .context("JWT_REFRESH_EXPIRE_SECONDS must be a valid i64")?,
            },
        })
    }
}
