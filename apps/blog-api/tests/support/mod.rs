//! blog-api 集成测试的公共辅助函数。
//!
//! 这里封装了应用构建、临时数据库初始化、认证会话创建与测试数据清理，
//! 以减少各测试文件中的重复样板代码。

use std::{
    ops::Deref,
    str::FromStr,
    sync::{Arc, Once, OnceLock},
};

use app_foundation::logging::init_tracing;
use app_testkit::request_json;
use axum::Router;
use blog_api::{
    app,
    config::{AppConfig, AuthConfig},
    db::rbac_repo,
    modules::rbac::keys::RoleKey,
    state::AppState,
};
use http::{Method, StatusCode};
use serde_json::json;
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    ConnectOptions, Executor, PgPool,
};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use tracing::{info, warn};
use uuid::Uuid;

static TEST_LOGGING_INIT: Once = Once::new();
static SHARED_TEST_DB: OnceLock<tokio::sync::OnceCell<Arc<SharedTestDatabase>>> = OnceLock::new();
static DB_TEST_PERMITS: OnceLock<Arc<Semaphore>> = OnceLock::new();
static TEST_DB_CLEANUP: OnceLock<(String, String)> = OnceLock::new();
static TEST_DB_ATEXIT_INIT: Once = Once::new();

fn init_test_logging() {
    TEST_LOGGING_INIT.call_once(init_tracing);
}

#[allow(dead_code)]
/// 测试过程中可复用的一组认证会话信息。
#[derive(Debug, Clone)]
pub struct AuthSession {
    /// 当前 refresh token 会话 ID。
    pub session_id: Uuid,
    /// 当前登录用户 ID。
    pub user_id: Uuid,
    /// access token。
    pub access_token: String,
    /// refresh token。
    pub refresh_token: String,
}

#[derive(Debug)]
struct SharedTestDatabase {
    database_url: String,
}

impl SharedTestDatabase {
    fn new(database_url: String) -> Self {
        Self { database_url }
    }
}

/// 测试期间使用的临时数据库句柄。
///
/// 同一个测试进程内会复用同一套临时数据库；最后一个引用释放时自动删库。
#[derive(Debug)]
pub struct TestDatabase {
    pool: PgPool,
    _permit: OwnedSemaphorePermit,
}

impl TestDatabase {
    fn new(pool: PgPool, permit: OwnedSemaphorePermit) -> Self {
        Self {
            pool,
            _permit: permit,
        }
    }
}

impl Deref for TestDatabase {
    type Target = PgPool;

    fn deref(&self) -> &Self::Target {
        &self.pool
    }
}

/// 构造测试使用的应用配置。
///
/// 默认会优先读取环境中的 `DATABASE_URL`，否则回退到本地开发数据库地址。
pub fn test_config() -> AppConfig {
    init_test_logging();
    dotenvy::from_filename(".env")
        .or_else(|_| dotenvy::from_filename("apps/blog-api/.env"))
        .ok();

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
pub async fn setup_app_with_db() -> Option<(Router, AppConfig, TestDatabase)> {
    let mut config = test_config();
    let permit = db_test_permit_pool()
        .clone()
        .acquire_owned()
        .await
        .expect("db test semaphore closed");
    let shared = match get_or_init_shared_test_db(&config.database_url).await {
        Some(shared) => shared,
        None => return None,
    };
    config.database_url = shared.database_url.clone();
    let pool = match PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await
    {
        Ok(pool) => pool,
        Err(err) => {
            warn!(error = %err, "skip test: cannot connect shared temporary database");
            return None;
        }
    };

    let state = AppState::new(pool.clone(), config.clone());
    let app = app::build_router(state);

    Some((app, config, TestDatabase::new(pool, permit)))
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
        session_id: login_body["data"]["token"]["session_id"]
            .as_str()
            .expect("missing session_id")
            .parse::<Uuid>()
            .expect("invalid session_id"),
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
        session_id: refresh_body["data"]["token"]["session_id"]
            .as_str()
            .expect("missing refreshed session_id")
            .parse::<Uuid>()
            .expect("invalid refreshed session_id"),
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
/// 直接在数据库中授予管理员角色，并通过 refresh 获取带新 claims 的会话。
///
/// 该辅助函数主要用于需要先以普通用户登录、再在测试中提升权限的场景。
pub async fn promote_to_admin(app: Router, db: &PgPool, session: AuthSession) -> AuthSession {
    let assigned = rbac_repo::assign_role_by_key(db, session.user_id, RoleKey::ADMIN)
        .await
        .expect("assign admin role failed");
    assert!(assigned, "admin role seed not found");

    refresh_session(app, &session.refresh_token).await
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

#[allow(dead_code)]
/// 删除指定博文，供集成测试清理数据使用。
pub async fn delete_posts(db: &PgPool, post_ids: &[Uuid]) {
    for post_id in post_ids {
        sqlx::query("DELETE FROM posts WHERE id = $1")
            .bind(*post_id)
            .execute(db)
            .await
            .expect("cleanup post failed");
    }
}

#[allow(dead_code)]
/// 将指定会话直接标记为已过期，供集成测试验证会话状态展示。
pub async fn expire_session(db: &PgPool, session_id: Uuid) {
    sqlx::query(
        r#"
        UPDATE refresh_tokens
        SET expires_at = NOW() - INTERVAL '1 minute'
        WHERE id = $1
        "#,
    )
    .bind(session_id)
    .execute(db)
    .await
    .expect("expire session failed");
}

fn build_temp_database_urls(database_url: &str) -> Result<(String, String, String), sqlx::Error> {
    let base_options = PgConnectOptions::from_str(database_url)?.disable_statement_logging();
    let database_name = format!("blog_api_test_{}", Uuid::new_v4().simple());
    let admin_database_url = base_options
        .clone()
        .database("postgres")
        .to_url_lossy()
        .to_string();
    let test_database_url = base_options
        .database(&database_name)
        .to_url_lossy()
        .to_string();

    Ok((admin_database_url, test_database_url, database_name))
}

fn shared_test_db_slot() -> &'static tokio::sync::OnceCell<Arc<SharedTestDatabase>> {
    SHARED_TEST_DB.get_or_init(tokio::sync::OnceCell::const_new)
}

fn db_test_permit_pool() -> &'static Arc<Semaphore> {
    DB_TEST_PERMITS.get_or_init(|| Arc::new(Semaphore::new(1)))
}

async fn get_or_init_shared_test_db(database_url: &str) -> Option<Arc<SharedTestDatabase>> {
    let shared = shared_test_db_slot()
        .get_or_try_init(|| async move {
            let (admin_database_url, test_database_url, database_name) =
                build_temp_database_urls(database_url)?;

            let admin_pool = PgPoolOptions::new()
                .max_connections(1)
                .connect(&admin_database_url)
                .await?;

            let create_sql = format!(r#"CREATE DATABASE "{database_name}""#);
            admin_pool.execute(create_sql.as_str()).await?;

            let pool = PgPoolOptions::new()
                .max_connections(5)
                .connect(&test_database_url)
                .await?;

            let db_identity = sqlx::query_as::<_, (Option<String>, Option<i32>, String, String)>(
                r#"
                    SELECT
                        inet_server_addr()::text,
                        inet_server_port(),
                        current_database(),
                        current_user
                    "#,
            )
            .fetch_one(&pool)
            .await?;

            info!(
                addr = ?db_identity.0,
                port = ?db_identity.1,
                database = %db_identity.2,
                user = %db_identity.3,
                "temporary test db connected"
            );

            sqlx::migrate!("./migrations").run(&pool).await?;

            let _ = TEST_DB_CLEANUP.set((admin_database_url.clone(), database_name.clone()));
            register_test_db_atexit();

            pool.close().await;

            Ok::<Arc<SharedTestDatabase>, sqlx::Error>(Arc::new(SharedTestDatabase::new(
                test_database_url,
            )))
        })
        .await;

    match shared {
        Ok(db) => Some(db.clone()),
        Err(err) => {
            warn!(error = %err, "skip test: cannot initialize shared temporary database");
            None
        }
    }
}

fn register_test_db_atexit() {
    TEST_DB_ATEXIT_INIT.call_once(|| unsafe {
        unsafe extern "C" {
            fn atexit(cb: extern "C" fn());
        }

        atexit(cleanup_shared_test_db);
    });
}

extern "C" fn cleanup_shared_test_db() {
    let Some((admin_database_url, database_name)) = TEST_DB_CLEANUP.get() else {
        return;
    };

    let admin_database_url = admin_database_url.clone();
    let database_name = database_name.clone();

    std::thread::spawn(move || {
        let runtime = tokio::runtime::Runtime::new().expect("create cleanup runtime failed");
        runtime.block_on(async move {
            let admin_pool = match PgPoolOptions::new()
                .max_connections(1)
                .connect(&admin_database_url)
                .await
            {
                Ok(pool) => pool,
                Err(err) => {
                    warn!(error = %err, database = %database_name, "skip cleanup: cannot connect admin database");
                    return;
                }
            };

            let terminate_sql = format!(
                r#"
                SELECT pg_terminate_backend(pid)
                FROM pg_stat_activity
                WHERE datname = '{database_name}'
                  AND pid <> pg_backend_pid()
                "#
            );
            if let Err(err) = admin_pool.execute(terminate_sql.as_str()).await {
                warn!(error = %err, database = %database_name, "cleanup warning: cannot terminate active connections");
            }

            let drop_sql = format!(r#"DROP DATABASE IF EXISTS "{database_name}""#);
            if let Err(err) = admin_pool.execute(drop_sql.as_str()).await {
                warn!(error = %err, database = %database_name, "cleanup warning: cannot drop temporary database");
            }
        });
    })
    .join()
    .expect("temporary database cleanup thread panicked");
}
