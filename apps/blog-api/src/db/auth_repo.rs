use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

pub async fn create_refresh_token(
    db: &PgPool,
    user_id: Uuid,
    refresh_token: &str,
    expires_at: OffsetDateTime,
) -> Result<(), sqlx::Error> {
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

    Ok(())
}

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
