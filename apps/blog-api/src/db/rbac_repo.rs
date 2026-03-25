use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

/// 给用户绑定指定角色（通过 role_key）。
/// 返回值表示该 role_key 是否存在。
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

/// 查询用户角色与权限，用于动态生成 JWT claims。
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
