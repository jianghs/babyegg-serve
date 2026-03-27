//! RBAC 管理接口相关的集成测试。
//!
//! 这里重点验证两类行为：
//! - RBAC 管理路由是否只允许管理员访问
//! - 管理员是否可以查询角色/权限并给用户分配角色

use app_testkit::{request_json, request_json_with_auth};
use http::{Method, StatusCode};
use serde_json::json;
use uuid::Uuid;

mod support;

#[tokio::test]
/// 验证 RBAC 管理接口会拒绝非管理员访问。
async fn rbac_management_routes_should_require_admin_role() {
    let Some((app, _config, db)) = support::setup_app_with_db().await else {
        return;
    };

    let email = format!("rbac-admin-{}@example.com", Uuid::new_v4());
    let password = "secret123";
    let session = support::register_and_login(app.clone(), "rbac-user", &email, password).await;

    let (forbidden_status, forbidden_body) = request_json_with_auth(
        app.clone(),
        Method::GET,
        "/rbac/roles",
        json!({}),
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
        Some("AUTH_FORBIDDEN_ROLE")
    );

    support::delete_users(&db, &[session.user_id]).await;
}

#[tokio::test]
/// 验证管理员可以管理用户角色并查看最新访问上下文。
async fn admin_should_manage_user_roles_and_access() {
    let Some((app, _config, db)) = support::setup_app_with_db().await else {
        return;
    };

    let admin_email = format!("admin-{}@example.com", Uuid::new_v4());
    let user_email = format!("managed-{}@example.com", Uuid::new_v4());
    let password = "secret123";

    let admin_session =
        support::register_and_login(app.clone(), "rbac-admin", &admin_email, password).await;
    let user_session =
        support::register_and_login(app.clone(), "rbac-managed", &user_email, password).await;
    let admin_user_id = admin_session.user_id;
    let managed_user_id = user_session.user_id;

    let (pre_assign_status, pre_assign_body) = request_json_with_auth(
        app.clone(),
        Method::GET,
        &format!("/rbac/users/{managed_user_id}"),
        json!({}),
        Some(&admin_session.access_token),
    )
    .await;
    assert_eq!(
        pre_assign_status,
        StatusCode::FORBIDDEN,
        "pre-assign body: {pre_assign_body}"
    );

    let refreshed_admin = support::promote_to_admin(app.clone(), &db, admin_session).await;

    let (roles_status, roles_body) = request_json_with_auth(
        app.clone(),
        Method::GET,
        "/rbac/roles",
        json!({}),
        Some(&refreshed_admin.access_token),
    )
    .await;
    assert_eq!(roles_status, StatusCode::OK, "roles body: {roles_body}");
    assert!(
        roles_body["data"]
            .as_array()
            .expect("roles data should be array")
            .iter()
            .any(|role| role["role_key"] == "admin"),
        "roles body: {roles_body}"
    );

    let (permissions_status, permissions_body) = request_json_with_auth(
        app.clone(),
        Method::GET,
        "/rbac/permissions",
        json!({}),
        Some(&refreshed_admin.access_token),
    )
    .await;
    assert_eq!(
        permissions_status,
        StatusCode::OK,
        "permissions body: {permissions_body}"
    );
    assert!(
        permissions_body["data"]
            .as_array()
            .expect("permissions data should be array")
            .iter()
            .any(|permission| permission["permission_key"] == "*"),
        "permissions body: {permissions_body}"
    );

    let (assign_status, assign_body) = request_json_with_auth(
        app.clone(),
        Method::POST,
        &format!("/rbac/users/{managed_user_id}/roles"),
        json!({ "role_key": "admin" }),
        Some(&refreshed_admin.access_token),
    )
    .await;
    assert_eq!(assign_status, StatusCode::OK, "assign body: {assign_body}");
    assert!(
        assign_body["data"]["roles"]
            .as_array()
            .expect("roles should be array")
            .iter()
            .any(|role| role["role_key"] == "admin"),
        "assign body: {assign_body}"
    );
    assert!(
        assign_body["data"]["scopes"]
            .as_array()
            .expect("scopes should be array")
            .iter()
            .any(|scope| scope == "*"),
        "assign body: {assign_body}"
    );

    let (user_access_status, user_access_body) = request_json_with_auth(
        app.clone(),
        Method::GET,
        &format!("/rbac/users/{managed_user_id}"),
        json!({}),
        Some(&refreshed_admin.access_token),
    )
    .await;
    assert_eq!(
        user_access_status,
        StatusCode::OK,
        "user access body: {user_access_body}"
    );
    let managed_user_id_str = managed_user_id.to_string();
    assert_eq!(
        user_access_body["data"]["user_id"].as_str(),
        Some(managed_user_id_str.as_str())
    );

    let (invalid_role_status, invalid_role_body) = request_json_with_auth(
        app.clone(),
        Method::POST,
        &format!("/rbac/users/{managed_user_id}/roles"),
        json!({ "role_key": "missing-role" }),
        Some(&refreshed_admin.access_token),
    )
    .await;
    assert_eq!(
        invalid_role_status,
        StatusCode::BAD_REQUEST,
        "invalid role body: {invalid_role_body}"
    );
    assert_eq!(
        invalid_role_body["error_code"].as_str(),
        Some("INVALID_PARAM")
    );

    let (missing_user_status, missing_user_body) = request_json_with_auth(
        app.clone(),
        Method::GET,
        &format!("/rbac/users/{}", Uuid::new_v4()),
        json!({}),
        Some(&refreshed_admin.access_token),
    )
    .await;
    assert_eq!(
        missing_user_status,
        StatusCode::NOT_FOUND,
        "missing user body: {missing_user_body}"
    );

    let (list_status, list_body) = request_json(
        app.clone(),
        Method::POST,
        "/auth/login",
        json!({
            "email": user_email,
            "password": password
        }),
    )
    .await;
    assert_eq!(list_status, StatusCode::OK, "login body: {list_body}");
    let managed_access_token = list_body["data"]["token"]["access_token"]
        .as_str()
        .expect("missing access token");

    let (managed_users_status, managed_users_body) = request_json_with_auth(
        app,
        Method::POST,
        "/users",
        json!({
            "name": "created-by-managed-admin",
            "email": format!("created-{}@example.com", Uuid::new_v4()),
            "password": "secret123"
        }),
        Some(managed_access_token),
    )
    .await;
    assert_eq!(
        managed_users_status,
        StatusCode::OK,
        "managed users body: {managed_users_body}"
    );

    let created_user_id = managed_users_body["data"]["id"]
        .as_str()
        .expect("missing created user id")
        .parse::<Uuid>()
        .expect("invalid created user id");
    support::delete_users(&db, &[created_user_id, admin_user_id, managed_user_id]).await;
}
