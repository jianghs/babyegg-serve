use app_foundation::ApiResponse;
use axum::{extract::State, Json};

use crate::{
    modules::external::{dto::HttpBinIpResponse, service},
    state::AppState,
};

pub async fn fetch_ip(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<HttpBinIpResponse>>, app_foundation::AppError> {
    let data = service::fetch_ip(&state).await?;
    Ok(Json(ApiResponse::ok(data)))
}
