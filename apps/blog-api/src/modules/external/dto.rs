use serde::{Deserialize, Serialize};

/// `httpbin /ip` 接口响应。
#[derive(Debug, Serialize, Deserialize)]
pub struct HttpBinIpResponse {
    /// 观察到的请求来源 IP。
    pub origin: String,
}
