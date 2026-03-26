use app_testkit::get_text;
use http::StatusCode;

mod support;

/// 这个测试验证基础健康检查接口是否正常工作。
/// 当前仍然会初始化一个数据库连接池，因为 AppState 里包含 db。
/// 后续如果你想彻底解耦 health 和 db，可以再把状态拆细一点。
#[tokio::test]
async fn health_endpoint_returns_ok() {
    let (app, _config) = support::setup_app_lazy();

    let (status, body) = get_text(app, "/health").await;

    assert_eq!(status, StatusCode::OK);
    assert!(body.contains("ok"));
}
