use std::fmt::Debug;

use axum::body::Body;
use http::{Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;

/// 发送一个 GET 请求，并返回状态码和响应文本。
pub async fn get_text<S>(app: S, uri: &str) -> (StatusCode, String)
where
    S: tower::Service<Request<Body>, Response = axum::response::Response> + Clone,
    S::Future: Send,
    S::Error: Debug,
{
    let response = app
        .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
        .await
        .unwrap();

    let status = response.status();
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let text = String::from_utf8(body.to_vec()).unwrap();

    (status, text)
}
