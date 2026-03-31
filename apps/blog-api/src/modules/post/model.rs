use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::OffsetDateTime;
use uuid::Uuid;

/// 数据库中的博文实体。
#[derive(Debug, Clone, Deserialize, FromRow)]
pub struct Post {
    /// 博文主键。
    pub id: Uuid,
    /// 标题。
    pub title: String,
    /// URL slug。
    pub slug: String,
    /// Markdown 正文。
    pub content_md: String,
    /// 是否已发布。
    pub published: bool,
    /// 作者用户 ID。
    pub author_id: Uuid,
    /// 创建时间。
    pub created_at: OffsetDateTime,
    /// 更新时间。
    pub updated_at: OffsetDateTime,
}

/// 对外返回的博文响应。
#[derive(Debug, Clone, Serialize)]
pub struct PostResponse {
    /// 博文主键字符串。
    pub id: String,
    /// 标题。
    pub title: String,
    /// URL slug。
    pub slug: String,
    /// Markdown 正文。
    pub content_md: String,
    /// 是否已发布。
    pub published: bool,
    /// 作者用户 ID。
    pub author_id: String,
    /// RFC3339 格式的创建时间。
    pub created_at: String,
    /// RFC3339 格式的更新时间。
    pub updated_at: String,
}

impl From<Post> for PostResponse {
    fn from(post: Post) -> Self {
        Self {
            id: post.id.to_string(),
            title: post.title,
            slug: post.slug,
            content_md: post.content_md,
            published: post.published,
            author_id: post.author_id.to_string(),
            created_at: format_datetime(post.created_at),
            updated_at: format_datetime(post.updated_at),
        }
    }
}

fn format_datetime(dt: OffsetDateTime) -> String {
    dt.format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_else(|_| dt.unix_timestamp().to_string())
}
