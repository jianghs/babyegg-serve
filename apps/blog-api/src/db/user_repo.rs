use sqlx::{PgPool, Postgres, QueryBuilder};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::modules::{
    identity::model::User,
    user::dto::{NormalizedUserListQuery, UserSortField},
};

/// 创建用户并返回完整用户记录。
///
/// 唯一约束冲突等数据库错误会原样以 `sqlx::Error` 向上抛出。
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

/// 按用户 ID 查询用户。
///
/// 未命中时返回 `Ok(None)`。
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
///
/// 未命中时返回 `Ok(None)`。
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
///
/// 支持对白名单字段排序，以及按 `name` / `email` 模糊搜索。
pub async fn list_users(
    db: &PgPool,
    query: &NormalizedUserListQuery,
) -> Result<Vec<User>, sqlx::Error> {
    let offset = (query.page - 1) * query.page_size;
    let mut builder = QueryBuilder::<Postgres>::new(
        "SELECT id, name, email, password_hash, created_at, updated_at FROM users",
    );

    push_user_filter_clause(&mut builder, query.filter.as_deref());

    builder.push(" ORDER BY ");
    match query.sort {
        UserSortField::CreatedAt => builder.push("created_at"),
        UserSortField::UpdatedAt => builder.push("updated_at"),
        UserSortField::Name => builder.push("name"),
        UserSortField::Email => builder.push("email"),
    };
    builder.push(match query.order {
        app_foundation::SortOrder::Asc => " ASC",
        app_foundation::SortOrder::Desc => " DESC",
    });
    builder.push(", id ASC");
    builder.push(" LIMIT ");
    builder.push_bind(query.page_size);
    builder.push(" OFFSET ");
    builder.push_bind(offset);

    builder.build_query_as::<User>().fetch_all(db).await
}

/// 查询用户总数。
///
/// 供分页响应计算总页数使用。
pub async fn count_users(db: &PgPool, filter: Option<&str>) -> Result<i64, sqlx::Error> {
    let mut builder = QueryBuilder::<Postgres>::new("SELECT COUNT(*) FROM users");
    push_user_filter_clause(&mut builder, filter);

    builder.build_query_scalar::<i64>().fetch_one(db).await
}

/// 更新用户名称并返回更新后的用户记录。
///
/// 未命中时返回 `Ok(None)`。
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

/// 删除用户，返回是否实际删除了记录。
///
/// 该返回值用于上游区分“删除成功”和“目标不存在”。
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

fn push_user_filter_clause(builder: &mut QueryBuilder<'_, Postgres>, filter: Option<&str>) {
    if let Some(filter) = filter {
        let like = format!("%{filter}%");
        builder.push(" WHERE (name ILIKE ");
        builder.push_bind(like.clone());
        builder.push(" OR email ILIKE ");
        builder.push_bind(like);
        builder.push(")");
    }
}
