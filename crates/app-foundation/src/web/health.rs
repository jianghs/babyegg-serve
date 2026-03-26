use axum::Json;
use serde::Serialize;

/// 健康检查响应体。
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    /// 服务状态，当前固定为 `ok`。
    pub status: &'static str,
}

/// 基础健康检查接口。
pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}
