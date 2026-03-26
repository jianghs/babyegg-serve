use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::modules::identity::model::User;

/// 创建用户。
pub async fn create_user(
    db: &PgPool,
    name: &str,
    email: &str,
    password_hash: &str,
) -> Result<User, sqlx::Error> {
    let id = Uuid::new_v4();
    let now = OffsetDateTime::now_utc();

    sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (id, name, email, password_hash, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, name, email, password_hash, created_at, updated_at
        "#,
    )
    .bind(id)
    .bind(name)
    .bind(email)
    .bind(password_hash)
    .bind(now)
    .bind(now)
    .fetch_one(db)
    .await
}

/// 按用户 ID 查询。
pub async fn get_user(db: &PgPool, id: Uuid) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as::<_, User>(
        r#"
        SELECT id, name, email, password_hash, created_at, updated_at
        FROM users
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(db)
    .await
}

/// 按邮箱查询用户。
pub async fn get_user_by_email(db: &PgPool, email: &str) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as::<_, User>(
        r#"
        SELECT id, name, email, password_hash, created_at, updated_at
        FROM users
        WHERE email = $1
        "#,
    )
    .bind(email)
    .fetch_optional(db)
    .await
}

/// 查询分页用户列表。
pub async fn list_users(db: &PgPool, page: i64, page_size: i64) -> Result<Vec<User>, sqlx::Error> {
    let offset = (page - 1) * page_size;

    sqlx::query_as::<_, User>(
        r#"
        SELECT id, name, email, password_hash, created_at, updated_at
        FROM users
        ORDER BY created_at DESC
        LIMIT $1 OFFSET $2
        "#,
    )
    .bind(page_size)
    .bind(offset)
    .fetch_all(db)
    .await
}

/// 查询用户总数。
pub async fn count_users(db: &PgPool) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM users
        "#,
    )
    .fetch_one(db)
    .await
}

/// 更新用户名称。
pub async fn update_user_name(
    db: &PgPool,
    id: Uuid,
    name: &str,
) -> Result<Option<User>, sqlx::Error> {
    let now = OffsetDateTime::now_utc();

    sqlx::query_as::<_, User>(
        r#"
        UPDATE users
        SET name = $1, updated_at = $2
        WHERE id = $3
        RETURNING id, name, email, password_hash, created_at, updated_at
        "#,
    )
    .bind(name)
    .bind(now)
    .bind(id)
    .fetch_optional(db)
    .await
}

/// 删除用户。
pub async fn delete_user(db: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        r#"
        DELETE FROM users
        WHERE id = $1
        "#,
    )
    .bind(id)
    .execute(db)
    .await?;

    Ok(result.rows_affected() > 0)
}
