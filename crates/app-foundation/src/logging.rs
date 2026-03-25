use tracing_subscriber::EnvFilter;

/// 初始化 tracing 日志系统。
/// 这里不强绑定具体业务服务，只负责统一日志格式与过滤规则。
pub fn init_tracing() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();
}
