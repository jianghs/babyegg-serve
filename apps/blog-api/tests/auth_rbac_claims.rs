//! 认证与 RBAC 相关的集成测试。
//!
//! 这里重点验证三类行为：
//! - 登录与 refresh 后的 JWT claims 是否与数据库中的 RBAC 状态保持一致
//! - 用户接口是否正确接入认证与 scope 校验
//! - 数据库中的 RBAC seed 是否与代码常量保持同步

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
/// 验证登录与 refresh 后签发的 claims 会动态反映 RBAC 变更。
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
/// 验证用户接口会要求认证，并按 scope 限制访问能力。
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
/// 验证数据库中的角色、权限与角色权限映射 seed 与代码常量一致。
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

#[tokio::test]
/// 验证用户列表支持排序、方向和模糊过滤。
async fn users_list_should_support_sort_order_and_filter() {
    let Some((app, _config, db)) = support::setup_app_with_db().await else {
        return;
    };

    let admin_session = support::register_and_login(
        app.clone(),
        "list-admin",
        &format!("list-admin-{}@example.com", Uuid::new_v4()),
        "secret123",
    )
    .await;
    let admin_session = support::promote_to_admin(app.clone(), &db, admin_session).await;

    let alpha_email = format!("alpha-filter-{}@example.com", Uuid::new_v4());
    let beta_email = format!("beta-filter-{}@example.com", Uuid::new_v4());
    let gamma_email = format!("gamma-filter-{}@example.com", Uuid::new_v4());

    let (create_alpha_status, create_alpha_body) = request_json_with_auth(
        app.clone(),
        Method::POST,
        "/users",
        json!({
            "name": "alpha-user",
            "email": alpha_email,
            "password": "secret123"
        }),
        Some(&admin_session.access_token),
    )
    .await;
    assert_eq!(
        create_alpha_status,
        StatusCode::OK,
        "create alpha body: {create_alpha_body}"
    );
    let alpha_id = create_alpha_body["data"]["id"]
        .as_str()
        .expect("missing alpha id")
        .parse::<Uuid>()
        .expect("invalid alpha id");

    let (create_beta_status, create_beta_body) = request_json_with_auth(
        app.clone(),
        Method::POST,
        "/users",
        json!({
            "name": "beta-user",
            "email": beta_email,
            "password": "secret123"
        }),
        Some(&admin_session.access_token),
    )
    .await;
    assert_eq!(
        create_beta_status,
        StatusCode::OK,
        "create beta body: {create_beta_body}"
    );
    let beta_id = create_beta_body["data"]["id"]
        .as_str()
        .expect("missing beta id")
        .parse::<Uuid>()
        .expect("invalid beta id");

    let (create_gamma_status, create_gamma_body) = request_json_with_auth(
        app.clone(),
        Method::POST,
        "/users",
        json!({
            "name": "gamma-user",
            "email": gamma_email,
            "password": "secret123"
        }),
        Some(&admin_session.access_token),
    )
    .await;
    assert_eq!(
        create_gamma_status,
        StatusCode::OK,
        "create gamma body: {create_gamma_body}"
    );
    let gamma_id = create_gamma_body["data"]["id"]
        .as_str()
        .expect("missing gamma id")
        .parse::<Uuid>()
        .expect("invalid gamma id");

    let (filtered_status, filtered_body) = request_empty_json_with_auth(
        app.clone(),
        Method::GET,
        "/users?sort=name&order=asc&filter=filter-",
        Some(&admin_session.access_token),
    )
    .await;
    assert_eq!(filtered_status, StatusCode::OK, "body: {filtered_body}");
    let filtered_items = filtered_body["data"]["items"]
        .as_array()
        .expect("items should be array");
    let filtered_names = filtered_items
        .iter()
        .map(|item| item["name"].as_str().expect("name should be string"))
        .collect::<Vec<_>>();
    assert_eq!(
        filtered_names,
        vec!["alpha-user", "beta-user", "gamma-user"]
    );
    assert_eq!(filtered_body["data"]["total"].as_i64(), Some(3));

    let (email_sorted_status, email_sorted_body) = request_empty_json_with_auth(
        app.clone(),
        Method::GET,
        "/users?sort=email&order=desc&filter=filter-",
        Some(&admin_session.access_token),
    )
    .await;
    assert_eq!(
        email_sorted_status,
        StatusCode::OK,
        "body: {email_sorted_body}"
    );
    let email_sorted_names = email_sorted_body["data"]["items"]
        .as_array()
        .expect("items should be array")
        .iter()
        .map(|item| item["name"].as_str().expect("name should be string"))
        .collect::<Vec<_>>();
    assert_eq!(
        email_sorted_names,
        vec!["gamma-user", "beta-user", "alpha-user"]
    );

    let (invalid_sort_status, invalid_sort_body) = request_empty_json_with_auth(
        app,
        Method::GET,
        "/users?sort=unknown",
        Some(&admin_session.access_token),
    )
    .await;
    assert_eq!(
        invalid_sort_status,
        StatusCode::BAD_REQUEST,
        "body: {invalid_sort_body}"
    );
    assert_eq!(
        invalid_sort_body["error_code"].as_str(),
        Some("INVALID_PARAM")
    );

    support::delete_users(&db, &[alpha_id, beta_id, gamma_id, admin_session.user_id]).await;
}
