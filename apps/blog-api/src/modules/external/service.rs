use app_foundation::{
    i18n::{translate, MessageKey},
    AppError,
};

use crate::{modules::external::dto::HttpBinIpResponse, state::AppState};

/// 调用外部 `httpbin` 服务获取当前出口 IP。
pub async fn fetch_ip(state: &AppState) -> Result<HttpBinIpResponse, AppError> {
    let locale = state.config.base.default_locale;
    let url = format!("{}/ip", state.config.httpbin_base_url.trim_end_matches('/'));

    let data = state
        .http_client
        .get(url)
        .send()
        .await
        .map_err(|_| {
            AppError::InternalWithMessage(
                translate(locale, MessageKey::InternalServerError).to_string(),
            )
        })?
        .error_for_status()
        .map_err(|_| {
            AppError::InternalWithMessage(
                translate(locale, MessageKey::InternalServerError).to_string(),
            )
        })?
        .json::<HttpBinIpResponse>()
        .await
        .map_err(|_| {
            AppError::InternalWithMessage(
                translate(locale, MessageKey::InternalServerError).to_string(),
            )
        })?;

    Ok(data)
}
