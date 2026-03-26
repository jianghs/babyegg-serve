use app_testkit::get_text;
use http::StatusCode;

mod support;

/// 验证基础健康检查接口可正常返回 `200 OK`。
///
/// 当前仍然会初始化一个惰性数据库连接池，因为 `AppState` 中包含数据库池字段。
#[tokio::test]
async fn health_endpoint_returns_ok() {
    let (app, _config) = support::setup_app_lazy();

    let (status, body) = get_text(app, "/health").await;

    assert_eq!(status, StatusCode::OK);
    assert!(body.contains("ok"));
}
