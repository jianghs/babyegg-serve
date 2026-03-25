use axum::{
    extract::Request,
    http::{HeaderName, HeaderValue},
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

pub const X_REQUEST_ID: &str = "x-request-id";

/// 为请求注入 request id。
/// 如果客户端已携带则透传，否则自动生成。
pub async fn inject_request_id(mut req: Request, next: Next) -> Response {
    let header_name = HeaderName::from_static(X_REQUEST_ID);

    let request_id = req
        .headers()
        .get(&header_name)
        .and_then(|v| v.to_str().ok())
        .map(ToString::to_string)
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    req.extensions_mut().insert(request_id.clone());

    let mut resp = next.run(req).await;
    if let Ok(value) = HeaderValue::from_str(&request_id) {
        resp.headers_mut().insert(header_name, value);
    }

    resp
}
