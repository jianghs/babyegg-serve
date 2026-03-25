use axum::{body::Body, Router};
use blog_api::{app, config::AppConfig, db::rbac_repo, modules::auth::jwt, state::AppState};
use http::{Method, Request, StatusCode};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;

async fn setup_app_with_db() -> Option<(Router, AppConfig, PgPool)> {
    let config = AppConfig {
        base: app_foundation::BaseConfig {
            host: "127.0.0.1".to_string(),
            port: 3000,
            default_locale: app_foundation::Locale::ZhCn,
        },
        database_url: std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:postgres@127.0.0.1:5432/blog_api".to_string()),
        httpbin_base_url: "https://httpbin.org".to_string(),
        jwt_secret: "change_me_in_production".to_string(),
        jwt_expire_seconds: 86400,
        jwt_refresh_expire_seconds: 604800,
    };

    let pool = match sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await
    {
        Ok(pool) => pool,
        Err(err) => {
            eprintln!("skip auth_rbac_claims test: cannot connect database: {err}");
            return None;
        }
    };

    if let Err(err) = sqlx::migrate!("./migrations").run(&pool).await {
        eprintln!("skip auth_rbac_claims test: cannot run migrations: {err}");
        return None;
    }

    let state = AppState::new(pool.clone(), config.clone());
    let app = app::build_router(state);
    Some((app, config, pool))
}

async fn request_json(
    app: Router,
    method: Method,
    uri: &str,
    payload: Value,
) -> (StatusCode, Value) {
    let req = Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json")
        .body(Body::from(payload.to_string()))
        .expect("failed to build request");

    let resp = app.oneshot(req).await.expect("request failed");
    let status = resp.status();
    let bytes = resp
        .into_body()
        .collect()
        .await
        .expect("read body failed")
        .to_bytes();
    let body: Value = serde_json::from_slice(&bytes).expect("response is not json");

    (status, body)
}

#[tokio::test]
async fn login_and_refresh_should_issue_dynamic_claims_from_rbac() {
    let Some((app, config, db)) = setup_app_with_db().await else {
        return;
    };

    let email = format!("rbac-{}@example.com", Uuid::new_v4());
    let password = "secret123";

    let (register_status, register_body) = request_json(
        app.clone(),
        Method::POST,
        "/auth/register",
        json!({
            "name": "rbac-user",
            "email": email,
            "password": password
        }),
    )
    .await;
    assert_eq!(
        register_status,
        StatusCode::OK,
        "register body: {register_body}"
    );

    let (login_status, login_body) = request_json(
        app.clone(),
        Method::POST,
        "/auth/login",
        json!({
            "email": email,
            "password": password
        }),
    )
    .await;
    assert_eq!(login_status, StatusCode::OK, "login body: {login_body}");

    let access_token = login_body["data"]["token"]["access_token"]
        .as_str()
        .expect("missing access_token");
    let refresh_token = login_body["data"]["token"]["refresh_token"]
        .as_str()
        .expect("missing refresh_token");
    let user_id = login_body["data"]["user"]["id"]
        .as_str()
        .expect("missing user.id")
        .parse::<Uuid>()
        .expect("invalid user.id");

    let claims = jwt::verify_token(access_token, &config.jwt_secret).expect("invalid access token");
    assert!(
        claims.roles.iter().any(|r| r == "user"),
        "claims: {:?}",
        claims
    );
    assert!(
        claims.scopes.iter().any(|s| s == "users:read"),
        "claims: {:?}",
        claims
    );
    assert!(
        !claims.scopes.iter().any(|s| s == "*"),
        "normal user should not have wildcard scope, claims: {:?}",
        claims
    );

    let assigned = rbac_repo::assign_role_by_key(&db, user_id, "admin")
        .await
        .expect("assign admin role failed");
    assert!(assigned, "admin role seed not found");

    let (refresh_status, refresh_body) = request_json(
        app.clone(),
        Method::POST,
        "/auth/refresh",
        json!({ "refresh_token": refresh_token }),
    )
    .await;
    assert_eq!(
        refresh_status,
        StatusCode::OK,
        "refresh body: {refresh_body}"
    );

    let refreshed_access_token = refresh_body["data"]["token"]["access_token"]
        .as_str()
        .expect("missing refreshed access_token");
    let refreshed_refresh_token = refresh_body["data"]["token"]["refresh_token"]
        .as_str()
        .expect("missing refreshed refresh_token");
    assert_ne!(refresh_token, refreshed_refresh_token);

    let refreshed_claims = jwt::verify_token(refreshed_access_token, &config.jwt_secret)
        .expect("invalid refreshed access token");
    assert!(
        refreshed_claims.roles.iter().any(|r| r == "admin"),
        "refreshed claims should include admin role: {:?}",
        refreshed_claims
    );
    assert!(
        refreshed_claims.scopes.iter().any(|s| s == "*"),
        "refreshed claims should include wildcard scope: {:?}",
        refreshed_claims
    );

    let (reuse_old_refresh_status, reuse_old_refresh_body) = request_json(
        app,
        Method::POST,
        "/auth/refresh",
        json!({ "refresh_token": refresh_token }),
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

    sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(user_id)
        .execute(&db)
        .await
        .expect("cleanup user failed");
}
