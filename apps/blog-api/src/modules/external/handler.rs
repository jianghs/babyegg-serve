use app_foundation::ApiResponse;
use axum::{extract::State, Json};

use crate::{
    modules::external::{dto::HttpBinIpResponse, service},
    state::AppState,
};

/// 处理查询出口 IP 请求。
pub async fn fetch_ip(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<HttpBinIpResponse>>, app_foundation::AppError> {
    let data = service::fetch_ip(&state).await?;
    Ok(Json(ApiResponse::ok(data)))
}
