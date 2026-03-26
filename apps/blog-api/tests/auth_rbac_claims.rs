use app_testkit::{request_empty_json_with_auth, request_json, request_json_with_auth};
use blog_api::{
    db::rbac_repo,
    modules::{
        auth::jwt,
        rbac::{
            catalog::{PERMISSION_KEYS, ROLE_KEYS, ROLE_PERMISSION_PAIRS},
            keys::{PermissionKey, RoleKey},
        },
    },
};
use http::{Method, StatusCode};
use serde_json::json;
use uuid::Uuid;

mod support;

#[tokio::test]
async fn login_and_refresh_should_issue_dynamic_claims_from_rbac() {
    let Some((app, config, db)) = support::setup_app_with_db().await else {
        return;
    };

    let email = format!("rbac-{}@example.com", Uuid::new_v4());
    let password = "secret123";
    let session = support::register_and_login(app.clone(), "rbac-user", &email, password).await;

    let claims = jwt::verify_token(&session.access_token, &config.auth.jwt_secret)
        .expect("invalid access token");
    assert!(
        claims.roles.iter().any(|r| r == RoleKey::USER),
        "claims: {:?}",
        claims
    );
    assert!(
        claims.scopes.iter().any(|s| s == PermissionKey::USERS_READ),
        "claims: {:?}",
        claims
    );
    assert!(
        !claims.scopes.iter().any(|s| s == PermissionKey::WILDCARD),
        "normal user should not have wildcard scope, claims: {:?}",
        claims
    );

    let assigned = rbac_repo::assign_role_by_key(&db, session.user_id, RoleKey::ADMIN)
        .await
        .expect("assign admin role failed");
    assert!(assigned, "admin role seed not found");

    let refreshed_session = support::refresh_session(app.clone(), &session.refresh_token).await;
    assert_ne!(session.refresh_token, refreshed_session.refresh_token);

    let refreshed_claims =
        jwt::verify_token(&refreshed_session.access_token, &config.auth.jwt_secret)
            .expect("invalid refreshed access token");
    assert!(
        refreshed_claims.roles.iter().any(|r| r == RoleKey::ADMIN),
        "refreshed claims should include admin role: {:?}",
        refreshed_claims
    );
    assert!(
        refreshed_claims
            .scopes
            .iter()
            .any(|s| s == PermissionKey::WILDCARD),
        "refreshed claims should include wildcard scope: {:?}",
        refreshed_claims
    );

    let (reuse_old_refresh_status, reuse_old_refresh_body) = request_json(
        app,
        Method::POST,
        "/auth/refresh",
        json!({ "refresh_token": session.refresh_token }),
    )
    .await;
    assert_eq!(
        reuse_old_refresh_status,
        StatusCode::BAD_REQUEST,
        "reuse old refresh body: {reuse_old_refresh_body}"
    );
    assert_eq!(
        reuse_old_refresh_body["error_code"].as_str(),
        Some("AUTH_INVALID_REFRESH_TOKEN")
    );

    support::delete_users(&db, &[session.user_id]).await;
}

#[tokio::test]
async fn users_routes_should_require_auth_and_enforce_scopes() {
    let Some((app, _config, db)) = support::setup_app_with_db().await else {
        return;
    };

    let email = format!("users-scope-{}@example.com", Uuid::new_v4());
    let password = "secret123";
    let session = support::register_and_login(app.clone(), "scope-user", &email, password).await;

    let (unauthorized_status, unauthorized_body) =
        request_empty_json_with_auth(app.clone(), Method::GET, "/users", None).await;
    assert_eq!(
        unauthorized_status,
        StatusCode::BAD_REQUEST,
        "unauthorized body: {unauthorized_body}"
    );
    assert_eq!(
        unauthorized_body["error_code"].as_str(),
        Some("AUTH_MISSING_AUTHORIZATION_HEADER")
    );

    let (list_status, list_body) = request_empty_json_with_auth(
        app.clone(),
        Method::GET,
        "/users",
        Some(&session.access_token),
    )
    .await;
    assert_eq!(list_status, StatusCode::OK, "list body: {list_body}");

    let (forbidden_status, forbidden_body) = request_json_with_auth(
        app.clone(),
        Method::POST,
        "/users",
        json!({
            "name": "another-user",
            "email": format!("admin-only-{}@example.com", Uuid::new_v4()),
            "password": "secret123"
        }),
        Some(&session.access_token),
    )
    .await;
    assert_eq!(
        forbidden_status,
        StatusCode::FORBIDDEN,
        "forbidden body: {forbidden_body}"
    );
    assert_eq!(
        forbidden_body["error_code"].as_str(),
        Some("AUTH_FORBIDDEN_SCOPE")
    );

    let assigned = rbac_repo::assign_role_by_key(&db, session.user_id, RoleKey::ADMIN)
        .await
        .expect("assign admin role failed");
    assert!(assigned, "admin role seed not found");

    let refreshed_session = support::refresh_session(app.clone(), &session.refresh_token).await;
    let create_email = format!("admin-created-{}@example.com", Uuid::new_v4());
    let (create_status, create_body) = request_json_with_auth(
        app.clone(),
        Method::POST,
        "/users",
        json!({
            "name": "admin-created",
            "email": create_email,
            "password": "secret123"
        }),
        Some(&refreshed_session.access_token),
    )
    .await;
    assert_eq!(create_status, StatusCode::OK, "create body: {create_body}");

    let created_user_id = create_body["data"]["id"]
        .as_str()
        .expect("missing created user id")
        .parse::<Uuid>()
        .expect("invalid created user id");

    support::delete_users(&db, &[created_user_id, session.user_id]).await;
}

#[tokio::test]
async fn rbac_seed_should_match_code_keys() {
    let Some((_app, _config, db)) = support::setup_app_with_db().await else {
        return;
    };

    let role_keys = rbac_repo::list_role_keys(&db)
        .await
        .expect("list role keys failed");
    assert_eq!(
        role_keys,
        ROLE_KEYS
            .iter()
            .map(|key| key.to_string())
            .collect::<Vec<_>>()
    );

    let permission_keys = rbac_repo::list_permission_keys(&db)
        .await
        .expect("list permission keys failed");
    assert_eq!(
        permission_keys,
        PERMISSION_KEYS
            .iter()
            .map(|key| key.to_string())
            .collect::<Vec<_>>()
    );

    let role_permissions = rbac_repo::list_role_permission_pairs(&db)
        .await
        .expect("list role permission pairs failed");
    assert_eq!(
        role_permissions,
        ROLE_PERMISSION_PAIRS
            .iter()
            .map(|(role, permission)| (role.to_string(), permission.to_string()))
            .collect::<Vec<_>>()
    );
}
