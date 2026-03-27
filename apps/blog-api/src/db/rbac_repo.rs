use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct RoleRecord {
    pub role_key: String,
    pub description: String,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct PermissionRecord {
    pub permission_key: String,
    pub description: String,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct UserRoleRecord {
    pub role_key: String,
    pub description: String,
}

/// 查询数据库中现存的全部角色键。
///
/// 主要用于校验 seed 或测试断言。
pub async fn list_role_keys(db: &PgPool) -> Result<Vec<String>, sqlx::Error> {
    sqlx::query_scalar::<_, String>(
        r#"
        SELECT role_key
        FROM roles
        ORDER BY role_key
        "#,
    )
    .fetch_all(db)
    .await
}

/// 查询数据库中全部角色及其描述。
pub async fn list_roles(db: &PgPool) -> Result<Vec<RoleRecord>, sqlx::Error> {
    sqlx::query_as::<_, RoleRecord>(
        r#"
        SELECT role_key, description
        FROM roles
        ORDER BY role_key
        "#,
    )
    .fetch_all(db)
    .await
}

/// 查询数据库中现存的全部权限键。
///
/// 主要用于校验 seed 或测试断言。
pub async fn list_permission_keys(db: &PgPool) -> Result<Vec<String>, sqlx::Error> {
    sqlx::query_scalar::<_, String>(
        r#"
        SELECT permission_key
        FROM permissions
        ORDER BY permission_key
        "#,
    )
    .fetch_all(db)
    .await
}

/// 查询数据库中全部权限及其描述。
pub async fn list_permissions(db: &PgPool) -> Result<Vec<PermissionRecord>, sqlx::Error> {
    sqlx::query_as::<_, PermissionRecord>(
        r#"
        SELECT permission_key, description
        FROM permissions
        ORDER BY permission_key
        "#,
    )
    .fetch_all(db)
    .await
}

/// 查询角色与权限的绑定关系。
///
/// 返回的元组结构为 `(role_key, permission_key)`。
pub async fn list_role_permission_pairs(db: &PgPool) -> Result<Vec<(String, String)>, sqlx::Error> {
    sqlx::query_as::<_, (String, String)>(
        r#"
        SELECT r.role_key, p.permission_key
        FROM role_permissions rp
        JOIN roles r ON r.id = rp.role_id
        JOIN permissions p ON p.id = rp.permission_id
        ORDER BY r.role_key, p.permission_key
        "#,
    )
    .fetch_all(db)
    .await
}

/// 查询某个角色对应的全部权限键。
pub async fn list_permissions_by_role_key(
    db: &PgPool,
    role_key: &str,
) -> Result<Vec<String>, sqlx::Error> {
    sqlx::query_scalar::<_, String>(
        r#"
        SELECT p.permission_key
        FROM role_permissions rp
        JOIN roles r ON r.id = rp.role_id
        JOIN permissions p ON p.id = rp.permission_id
        WHERE r.role_key = $1
        ORDER BY p.permission_key
        "#,
    )
    .bind(role_key)
    .fetch_all(db)
    .await
}

/// 给用户绑定指定角色（通过 role_key）。
///
/// 返回值语义：
/// - `Ok(true)`：角色存在，且绑定操作已执行或因冲突被安全忽略
/// - `Ok(false)`：指定 `role_key` 不存在
pub async fn assign_role_by_key(
    db: &PgPool,
    user_id: Uuid,
    role_key: &str,
) -> Result<bool, sqlx::Error> {
    let role_id = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT id
        FROM roles
        WHERE role_key = $1
        LIMIT 1
        "#,
    )
    .bind(role_key)
    .fetch_optional(db)
    .await?;

    let Some(role_id) = role_id else {
        return Ok(false);
    };

    sqlx::query(
        r#"
        INSERT INTO user_roles (user_id, role_id, created_at)
        VALUES ($1, $2, $3)
        ON CONFLICT (user_id, role_id) DO NOTHING
        "#,
    )
    .bind(user_id)
    .bind(role_id)
    .bind(OffsetDateTime::now_utc())
    .execute(db)
    .await?;

    Ok(true)
}

/// 查询用户显式绑定的角色及描述。
pub async fn list_user_roles(
    db: &PgPool,
    user_id: Uuid,
) -> Result<Vec<UserRoleRecord>, sqlx::Error> {
    sqlx::query_as::<_, UserRoleRecord>(
        r#"
        SELECT r.role_key, r.description
        FROM user_roles ur
        JOIN roles r ON r.id = ur.role_id
        WHERE ur.user_id = $1
        ORDER BY r.role_key
        "#,
    )
    .bind(user_id)
    .fetch_all(db)
    .await
}

/// 查询用户角色与权限，用于动态生成 JWT claims。
///
/// 返回值中的两个列表都已按键名排序，便于测试与比较。
pub async fn get_user_roles_and_scopes(
    db: &PgPool,
    user_id: Uuid,
) -> Result<(Vec<String>, Vec<String>), sqlx::Error> {
    let roles = sqlx::query_scalar::<_, String>(
        r#"
        SELECT DISTINCT r.role_key
        FROM user_roles ur
        JOIN roles r ON r.id = ur.role_id
        WHERE ur.user_id = $1
        ORDER BY r.role_key
        "#,
    )
    .bind(user_id)
    .fetch_all(db)
    .await?;

    let scopes = sqlx::query_scalar::<_, String>(
        r#"
        SELECT DISTINCT p.permission_key
        FROM user_roles ur
        JOIN role_permissions rp ON rp.role_id = ur.role_id
        JOIN permissions p ON p.id = rp.permission_id
        WHERE ur.user_id = $1
        ORDER BY p.permission_key
        "#,
    )
    .bind(user_id)
    .fetch_all(db)
    .await?;

    Ok((roles, scopes))
}
