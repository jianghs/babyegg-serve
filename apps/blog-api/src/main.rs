use anyhow::Context;
use app_foundation::i18n::{translate, MessageKey};
use app_foundation::logging::init_tracing;
use sqlx::PgPool;
use tracing::info;

use blog_api::{app, config::AppConfig, state::AppState};

/// 应用启动入口。
///
/// 启动流程：
/// - 读取 `.env`
/// - 初始化日志
/// - 加载配置并连接数据库
/// - 按需执行迁移
/// - 构建路由并启动 HTTP 服务
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    init_tracing();

    let config = AppConfig::from_env().context("failed to load config")?;

    let pool = PgPool::connect(&config.database_url)
        .await
        .context("failed to connect database")?;

    let skip_migrations = std::env::var("SKIP_MIGRATIONS")
        .map(|v| {
            matches!(
                v.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(false);

    if skip_migrations {
        info!("skip database migrations on startup");
    } else {
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .context("failed to run migrations")?;
    }

    let state = AppState::new(pool, config.clone());
    let app = app::build_router(state);

    let listener = tokio::net::TcpListener::bind((&config.base.host[..], config.base.port))
        .await
        .context("failed to bind tcp listener")?;

    info!(
        message = translate(config.base.default_locale, MessageKey::ServerStarted),
        host = %config.base.host,
        port = config.base.port
    );

    axum::serve(listener, app).await.context("server error")?;

    Ok(())
}
