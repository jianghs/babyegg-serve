use axum::Json;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
}

/// 基础健康检查接口。
pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}
