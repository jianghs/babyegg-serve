//! blog-api 集成测试的公共辅助函数。
//!
//! 这里封装了应用构建、数据库初始化、认证会话创建与测试数据清理，
//! 以减少各测试文件中的重复样板代码。

use app_testkit::request_json;
use axum::Router;
use blog_api::{
    app,
    config::{AppConfig, AuthConfig},
    state::AppState,
};
use http::{Method, StatusCode};
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AuthSession {
    /// 当前登录用户 ID。
    pub user_id: Uuid,
    /// access token。
    pub access_token: String,
    /// refresh token。
    pub refresh_token: String,
}

/// 构造测试使用的应用配置。
///
/// 默认会优先读取环境中的 `DATABASE_URL`，否则回退到本地开发数据库地址。
pub fn test_config() -> AppConfig {
    AppConfig {
        base: app_foundation::BaseConfig {
            host: "127.0.0.1".to_string(),
            port: 3000,
            default_locale: app_foundation::Locale::ZhCn,
        },
        database_url: std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:postgres@127.0.0.1:5432/blog_api".to_string()),
        httpbin_base_url: "https://httpbin.org".to_string(),
        auth: AuthConfig {
            jwt_secret: "change_me_in_production".to_string(),
            access_token_ttl_seconds: 86400,
            refresh_token_ttl_seconds: 604800,
        },
    }
}

#[allow(dead_code)]
/// 使用惰性数据库连接构造应用，适合不访问数据库的测试。
///
/// 该函数不会真正连接数据库，因此适合 health 等不依赖数据库 IO 的测试。
pub fn setup_app_lazy() -> (Router, AppConfig) {
    let config = test_config();
    let pool = sqlx::postgres::PgPoolOptions::new()
        .connect_lazy(&config.database_url)
        .expect("failed to create lazy test database pool");

    let state = AppState::new(pool, config.clone());
    let app = app::build_router(state);

    (app, config)
}

#[allow(dead_code)]
/// 连接测试数据库、执行迁移并返回可用应用。
///
/// 若数据库不可用，则返回 `None` 以便测试自行跳过。
pub async fn setup_app_with_db() -> Option<(Router, AppConfig, PgPool)> {
    let config = test_config();

    let pool = match sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await
    {
        Ok(pool) => pool,
        Err(err) => {
            eprintln!("skip test: cannot connect database: {err}");
            return None;
        }
    };

    if let Err(err) = sqlx::migrate!("./migrations").run(&pool).await {
        eprintln!("skip test: cannot run migrations: {err}");
        return None;
    }

    let state = AppState::new(pool.clone(), config.clone());
    let app = app::build_router(state);

    Some((app, config, pool))
}

#[allow(dead_code)]
/// 通过注册并登录流程创建一组可直接使用的认证会话。
///
/// 返回值中同时包含用户 ID、access token 和 refresh token，
/// 便于后续继续测试受保护接口和续签流程。
pub async fn register_and_login(
    app: Router,
    name: &str,
    email: &str,
    password: &str,
) -> AuthSession {
    let (register_status, register_body) = request_json(
        app.clone(),
        Method::POST,
        "/auth/register",
        json!({
            "name": name,
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
        app,
        Method::POST,
        "/auth/login",
        json!({
            "email": email,
            "password": password
        }),
    )
    .await;
    assert_eq!(login_status, StatusCode::OK, "login body: {login_body}");

    AuthSession {
        user_id: login_body["data"]["user"]["id"]
            .as_str()
            .expect("missing user.id")
            .parse::<Uuid>()
            .expect("invalid user.id"),
        access_token: login_body["data"]["token"]["access_token"]
            .as_str()
            .expect("missing access_token")
            .to_string(),
        refresh_token: login_body["data"]["token"]["refresh_token"]
            .as_str()
            .expect("missing refresh_token")
            .to_string(),
    }
}

#[allow(dead_code)]
/// 使用 refresh token 刷新会话并返回新的认证结果。
///
/// 该辅助函数默认断言刷新接口返回 `200 OK`。
pub async fn refresh_session(app: Router, refresh_token: &str) -> AuthSession {
    let (refresh_status, refresh_body) = request_json(
        app,
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

    AuthSession {
        user_id: refresh_body["data"]["user"]["id"]
            .as_str()
            .expect("missing user.id")
            .parse::<Uuid>()
            .expect("invalid user.id"),
        access_token: refresh_body["data"]["token"]["access_token"]
            .as_str()
            .expect("missing refreshed access_token")
            .to_string(),
        refresh_token: refresh_body["data"]["token"]["refresh_token"]
            .as_str()
            .expect("missing refreshed refresh_token")
            .to_string(),
    }
}

#[allow(dead_code)]
/// 删除指定用户，供集成测试清理数据使用。
///
/// 当前只删除 `users` 表中的记录，依赖数据库侧的级联或约束来处理关联数据。
pub async fn delete_users(db: &PgPool, user_ids: &[Uuid]) {
    for user_id in user_ids {
        sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(*user_id)
            .execute(db)
            .await
            .expect("cleanup user failed");
    }
}
