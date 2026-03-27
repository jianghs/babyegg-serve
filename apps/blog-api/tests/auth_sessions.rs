use app_testkit::{request_empty_json_with_auth, request_json};
use http::{Method, StatusCode};
use serde_json::json;
use uuid::Uuid;

mod support;

#[tokio::test]
async fn session_routes_should_require_auth() {
    let (app, _config) = support::setup_app_lazy();

    let (status, body) =
        request_empty_json_with_auth(app, Method::GET, "/auth/sessions", None).await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "body: {body}");
    assert_eq!(
        body["error_code"].as_str(),
        Some("AUTH_MISSING_AUTHORIZATION_HEADER")
    );
}

#[tokio::test]
async fn user_should_list_and_revoke_sessions() {
    let Some((app, _config, db)) = support::setup_app_with_db().await else {
        return;
    };

    let email = format!("session-{}@example.com", Uuid::new_v4());
    let password = "secret123";
    let session = support::register_and_login(app.clone(), "session-user", &email, password).await;

    let session_id = request_json(
        app.clone(),
        Method::POST,
        "/auth/login",
        json!({
            "email": email,
            "password": password
        }),
    )
    .await
    .1["data"]["token"]["session_id"]
        .as_str()
        .expect("missing session id")
        .parse::<Uuid>()
        .expect("invalid session id");

    let (list_status, list_body) = request_empty_json_with_auth(
        app.clone(),
        Method::GET,
        "/auth/sessions",
        Some(&session.access_token),
    )
    .await;
    assert_eq!(list_status, StatusCode::OK, "list body: {list_body}");
    assert!(
        list_body["data"]
            .as_array()
            .expect("sessions should be array")
            .iter()
            .any(|item| item["id"] == session_id.to_string()),
        "list body: {list_body}"
    );

    let (revoke_status, revoke_body) = request_empty_json_with_auth(
        app.clone(),
        Method::DELETE,
        &format!("/auth/sessions/{session_id}"),
        Some(&session.access_token),
    )
    .await;
    assert_eq!(revoke_status, StatusCode::OK, "revoke body: {revoke_body}");

    let (refresh_status, refresh_body) = request_json(
        app.clone(),
        Method::POST,
        "/auth/refresh",
        json!({ "refresh_token": session.refresh_token }),
    )
    .await;
    assert_eq!(
        refresh_status,
        StatusCode::BAD_REQUEST,
        "refresh body: {refresh_body}"
    );

    let (revoke_missing_status, revoke_missing_body) = request_empty_json_with_auth(
        app.clone(),
        Method::DELETE,
        &format!("/auth/sessions/{session_id}"),
        Some(&session.access_token),
    )
    .await;
    assert_eq!(
        revoke_missing_status,
        StatusCode::BAD_REQUEST,
        "revoke missing body: {revoke_missing_body}"
    );

    support::delete_users(&db, &[session.user_id]).await;
}

#[tokio::test]
async fn revoke_all_sessions_should_invalidate_all_refresh_tokens() {
    let Some((app, _config, db)) = support::setup_app_with_db().await else {
        return;
    };

    let email = format!("session-all-{}@example.com", Uuid::new_v4());
    let password = "secret123";
    let session = support::register_and_login(app.clone(), "session-user", &email, password).await;
    let second_login = request_json(
        app.clone(),
        Method::POST,
        "/auth/login",
        json!({
            "email": email,
            "password": password
        }),
    )
    .await;
    assert_eq!(
        second_login.0,
        StatusCode::OK,
        "login body: {}",
        second_login.1
    );
    let second_refresh_token = second_login.1["data"]["token"]["refresh_token"]
        .as_str()
        .expect("missing refresh token")
        .to_string();

    let (revoke_all_status, revoke_all_body) = request_empty_json_with_auth(
        app.clone(),
        Method::POST,
        "/auth/sessions/revoke-all",
        Some(&session.access_token),
    )
    .await;
    assert_eq!(
        revoke_all_status,
        StatusCode::OK,
        "revoke all body: {revoke_all_body}"
    );
    assert_eq!(
        revoke_all_body["data"]["revoked_sessions"].as_u64(),
        Some(2)
    );

    for refresh_token in [&session.refresh_token, second_refresh_token.as_str()] {
        let (refresh_status, refresh_body) = request_json(
            app.clone(),
            Method::POST,
            "/auth/refresh",
            json!({ "refresh_token": refresh_token }),
        )
        .await;
        assert_eq!(
            refresh_status,
            StatusCode::BAD_REQUEST,
            "refresh body: {refresh_body}"
        );
    }

    support::delete_users(&db, &[session.user_id]).await;
}
