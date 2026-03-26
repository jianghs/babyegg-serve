use std::str::FromStr;

use anyhow::{Context, Result};

use crate::locale::Locale;

/// 基础配置。
/// 这里放多个业务服务都可能需要的公共配置项。
#[derive(Debug, Clone)]
pub struct BaseConfig {
    /// 服务监听地址。
    pub host: String,
    /// 服务监听端口。
    pub port: u16,
    /// 默认语言区域。
    pub default_locale: Locale,
}

impl BaseConfig {
    /// 从环境变量加载基础配置。
    pub fn from_env() -> Result<Self> {
        let locale = std::env::var("DEFAULT_LOCALE").unwrap_or_else(|_| "zh-CN".to_string());

        Ok(Self {
            host: std::env::var("APP_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            port: std::env::var("APP_PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse::<u16>()
                .context("APP_PORT must be a valid u16")?,
            default_locale: Locale::from_str(&locale).unwrap_or(Locale::ZhCn),
        })
    }
}
