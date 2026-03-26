use std::fmt::Debug;

use axum::body::Body;
use http::{Method, Request, StatusCode};
use http_body_util::BodyExt;
use serde_json::Value;
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

pub async fn request_json<S>(
    app: S,
    method: Method,
    uri: &str,
    payload: Value,
) -> (StatusCode, Value)
where
    S: tower::Service<Request<Body>, Response = axum::response::Response> + Clone,
    S::Future: Send,
    S::Error: Debug,
{
    request_json_with_auth(app, method, uri, payload, None).await
}

pub async fn request_json_with_auth<S>(
    app: S,
    method: Method,
    uri: &str,
    payload: Value,
    bearer_token: Option<&str>,
) -> (StatusCode, Value)
where
    S: tower::Service<Request<Body>, Response = axum::response::Response> + Clone,
    S::Future: Send,
    S::Error: Debug,
{
    let request = Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json");

    let request = if let Some(token) = bearer_token {
        request.header("authorization", format!("Bearer {token}"))
    } else {
        request
    }
    .body(Body::from(payload.to_string()))
    .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let status = response.status();
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let value = serde_json::from_slice(&body).unwrap();

    (status, value)
}

pub async fn request_empty_json_with_auth<S>(
    app: S,
    method: Method,
    uri: &str,
    bearer_token: Option<&str>,
) -> (StatusCode, Value)
where
    S: tower::Service<Request<Body>, Response = axum::response::Response> + Clone,
    S::Future: Send,
    S::Error: Debug,
{
    let request = Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json");

    let request = if let Some(token) = bearer_token {
        request.header("authorization", format!("Bearer {token}"))
    } else {
        request
    }
    .body(Body::empty())
    .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let status = response.status();
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let value = serde_json::from_slice(&body).unwrap();

    (status, value)
}
