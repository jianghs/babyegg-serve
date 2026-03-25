use app_testkit::get_text;
use blog_api::{app, config::AppConfig, state::AppState};
use http::StatusCode;

/// 这个测试验证基础健康检查接口是否正常工作。
/// 当前仍然会初始化一个数据库连接池，因为 AppState 里包含 db。
/// 后续如果你想彻底解耦 health 和 db，可以再把状态拆细一点。
#[tokio::test]
async fn health_endpoint_returns_ok() {
    let config = AppConfig {
        base: app_foundation::BaseConfig {
            host: "127.0.0.1".to_string(),
            port: 3000,
            default_locale: app_foundation::Locale::ZhCn,
        },
        database_url: "postgres://postgres:postgres@127.0.0.1:5432/blog_api".to_string(),
        httpbin_base_url: "https://httpbin.org".to_string(),
        jwt_secret: "change_me_in_production".to_string(),
        jwt_expire_seconds: 86400,
    };

    let pool = sqlx::postgres::PgPoolOptions::new()
        .connect_lazy(&config.database_url)
        .expect("failed to create lazy test database pool");

    let state = AppState::new(pool, config);
    let app = app::build_router(state);

    let (status, body) = get_text(app, "/health").await;

    assert_eq!(status, StatusCode::OK);
    assert!(body.contains("ok"));
}
