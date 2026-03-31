use sqlx::{PgPool, Postgres, QueryBuilder};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::modules::post::{
    dto::{NormalizedPostListQuery, PostSortField},
    model::Post,
};

/// 创建博文并返回完整记录。
pub async fn create_post(
    db: &PgPool,
    author_id: Uuid,
    title: &str,
    slug: &str,
    content_md: &str,
    published: bool,
) -> Result<Post, sqlx::Error> {
    let id = Uuid::new_v4();
    let now = OffsetDateTime::now_utc();

    sqlx::query_as::<_, Post>(
        r#"
        INSERT INTO posts (id, title, slug, content_md, published, author_id, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING id, title, slug, content_md, published, author_id, created_at, updated_at
        "#,
    )
    .bind(id)
    .bind(title)
    .bind(slug)
    .bind(content_md)
    .bind(published)
    .bind(author_id)
    .bind(now)
    .bind(now)
    .fetch_one(db)
    .await
}

/// 按 ID 查询博文。
pub async fn get_post(db: &PgPool, id: Uuid) -> Result<Option<Post>, sqlx::Error> {
    sqlx::query_as::<_, Post>(
        r#"
        SELECT id, title, slug, content_md, published, author_id, created_at, updated_at
        FROM posts
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(db)
    .await
}

/// 按 slug 查询博文。
pub async fn get_post_by_slug(db: &PgPool, slug: &str) -> Result<Option<Post>, sqlx::Error> {
    sqlx::query_as::<_, Post>(
        r#"
        SELECT id, title, slug, content_md, published, author_id, created_at, updated_at
        FROM posts
        WHERE slug = $1
        "#,
    )
    .bind(slug)
    .fetch_optional(db)
    .await
}

/// 查询分页博文列表。
pub async fn list_posts(
    db: &PgPool,
    query: &NormalizedPostListQuery,
) -> Result<Vec<Post>, sqlx::Error> {
    let offset = (query.page - 1) * query.page_size;
    let mut builder = QueryBuilder::<Postgres>::new(
        "SELECT id, title, slug, content_md, published, author_id, created_at, updated_at FROM posts",
    );

    push_post_filter_clause(&mut builder, query.filter.as_deref(), query.published);

    builder.push(" ORDER BY ");
    match query.sort {
        PostSortField::CreatedAt => builder.push("created_at"),
        PostSortField::UpdatedAt => builder.push("updated_at"),
        PostSortField::Title => builder.push("title"),
        PostSortField::Slug => builder.push("slug"),
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

    builder.build_query_as::<Post>().fetch_all(db).await
}

/// 查询博文总数。
pub async fn count_posts(
    db: &PgPool,
    filter: Option<&str>,
    published: Option<bool>,
) -> Result<i64, sqlx::Error> {
    let mut builder = QueryBuilder::<Postgres>::new("SELECT COUNT(*) FROM posts");
    push_post_filter_clause(&mut builder, filter, published);
    builder.build_query_scalar::<i64>().fetch_one(db).await
}

/// 更新博文并返回更新后的记录。
pub async fn update_post(
    db: &PgPool,
    id: Uuid,
    title: &str,
    slug: &str,
    content_md: &str,
    published: bool,
) -> Result<Option<Post>, sqlx::Error> {
    let now = OffsetDateTime::now_utc();

    sqlx::query_as::<_, Post>(
        r#"
        UPDATE posts
        SET title = $1, slug = $2, content_md = $3, published = $4, updated_at = $5
        WHERE id = $6
        RETURNING id, title, slug, content_md, published, author_id, created_at, updated_at
        "#,
    )
    .bind(title)
    .bind(slug)
    .bind(content_md)
    .bind(published)
    .bind(now)
    .bind(id)
    .fetch_optional(db)
    .await
}

/// 删除博文。
pub async fn delete_post(db: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        r#"
        DELETE FROM posts
        WHERE id = $1
        "#,
    )
    .bind(id)
    .execute(db)
    .await?;

    Ok(result.rows_affected() > 0)
}

fn push_post_filter_clause(
    builder: &mut QueryBuilder<'_, Postgres>,
    filter: Option<&str>,
    published: Option<bool>,
) {
    let mut has_where = false;

    if let Some(filter) = filter {
        let like = format!("%{filter}%");
        builder.push(" WHERE (title ILIKE ");
        builder.push_bind(like.clone());
        builder.push(" OR slug ILIKE ");
        builder.push_bind(like);
        builder.push(")");
        has_where = true;
    }

    if let Some(published) = published {
        if has_where {
            builder.push(" AND published = ");
        } else {
            builder.push(" WHERE published = ");
        }
        builder.push_bind(published);
    }
}
