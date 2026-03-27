use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct SessionRecord {
    pub id: Uuid,
    pub user_id: Uuid,
    pub refresh_token: String,
    pub expires_at: OffsetDateTime,
    pub created_at: OffsetDateTime,
    pub revoked_at: Option<OffsetDateTime>,
}

/// 持久化 refresh token 会话记录。
///
/// 该仓储只负责写入会话数据，不在此处判断业务合法性。
pub async fn create_refresh_token(
    db: &PgPool,
    user_id: Uuid,
    refresh_token: &str,
    expires_at: OffsetDateTime,
) -> Result<Uuid, sqlx::Error> {
    let id = Uuid::new_v4();
    let now = OffsetDateTime::now_utc();

    sqlx::query(
        r#"
        INSERT INTO refresh_tokens (id, user_id, refresh_token, expires_at, created_at)
        VALUES ($1, $2, $3, $4, $5)
        "#,
    )
    .bind(id)
    .bind(user_id)
    .bind(refresh_token)
    .bind(expires_at)
    .bind(now)
    .execute(db)
    .await?;

    Ok(id)
}

/// 根据 refresh token 查询仍然有效的用户 ID。
///
/// “有效”定义为：
/// - `revoked_at IS NULL`
/// - `expires_at > now`
pub async fn find_valid_refresh_token_user_id(
    db: &PgPool,
    refresh_token: &str,
) -> Result<Option<Uuid>, sqlx::Error> {
    let now = OffsetDateTime::now_utc();

    sqlx::query_scalar::<_, Uuid>(
        r#"
        SELECT user_id
        FROM refresh_tokens
        WHERE refresh_token = $1
          AND revoked_at IS NULL
          AND expires_at > $2
        LIMIT 1
        "#,
    )
    .bind(refresh_token)
    .bind(now)
    .fetch_optional(db)
    .await
}

/// 撤销 refresh token，返回是否成功命中一条未撤销记录。
///
/// 若返回 `false`，通常表示 token 不存在，或此前已经被撤销。
pub async fn revoke_refresh_token(db: &PgPool, refresh_token: &str) -> Result<bool, sqlx::Error> {
    let now = OffsetDateTime::now_utc();

    let result = sqlx::query(
        r#"
        UPDATE refresh_tokens
        SET revoked_at = $1
        WHERE refresh_token = $2
          AND revoked_at IS NULL
        "#,
    )
    .bind(now)
    .bind(refresh_token)
    .execute(db)
    .await?;

    Ok(result.rows_affected() > 0)
}

/// 列出指定用户的全部 refresh token 会话。
pub async fn list_sessions_by_user_id(
    db: &PgPool,
    user_id: Uuid,
) -> Result<Vec<SessionRecord>, sqlx::Error> {
    sqlx::query_as::<_, SessionRecord>(
        r#"
        SELECT id, user_id, refresh_token, expires_at, created_at, revoked_at
        FROM refresh_tokens
        WHERE user_id = $1
        ORDER BY created_at DESC
        "#,
    )
    .bind(user_id)
    .fetch_all(db)
    .await
}

/// 撤销指定用户的某个会话。
pub async fn revoke_session_by_id(
    db: &PgPool,
    user_id: Uuid,
    session_id: Uuid,
) -> Result<bool, sqlx::Error> {
    let now = OffsetDateTime::now_utc();

    let result = sqlx::query(
        r#"
        UPDATE refresh_tokens
        SET revoked_at = $1
        WHERE id = $2
          AND user_id = $3
          AND revoked_at IS NULL
        "#,
    )
    .bind(now)
    .bind(session_id)
    .bind(user_id)
    .execute(db)
    .await?;

    Ok(result.rows_affected() > 0)
}

/// 撤销指定用户的全部有效会话。
pub async fn revoke_all_sessions_by_user_id(
    db: &PgPool,
    user_id: Uuid,
) -> Result<u64, sqlx::Error> {
    let now = OffsetDateTime::now_utc();

    let result = sqlx::query(
        r#"
        UPDATE refresh_tokens
        SET revoked_at = $1
        WHERE user_id = $2
          AND revoked_at IS NULL
        "#,
    )
    .bind(now)
    .bind(user_id)
    .execute(db)
    .await?;

    Ok(result.rows_affected())
}
