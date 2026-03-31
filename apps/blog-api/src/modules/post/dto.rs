use serde::Deserialize;

use app_foundation::{
    AppError, ErrorCode, Locale, PageResponse, SortOrder, ValidationDetail, ValidationReason,
};

use crate::modules::post::model::PostResponse;

/// 创建博文请求。
#[derive(Debug, Deserialize)]
pub struct CreatePostRequest {
    /// 标题。
    pub title: String,
    /// URL slug。
    pub slug: String,
    /// Markdown 正文。
    pub content_md: String,
    /// 是否发布。
    pub published: Option<bool>,
}

impl CreatePostRequest {
    /// 校验创建博文请求。
    pub fn validate(&self, locale: Locale) -> Result<(), AppError> {
        validate_post_fields(locale, &self.title, &self.slug, &self.content_md)
    }
}

/// 更新博文请求。
#[derive(Debug, Deserialize)]
pub struct UpdatePostRequest {
    /// 标题。
    pub title: String,
    /// URL slug。
    pub slug: String,
    /// Markdown 正文。
    pub content_md: String,
    /// 是否发布。
    pub published: bool,
}

impl UpdatePostRequest {
    /// 校验更新博文请求。
    pub fn validate(&self, locale: Locale) -> Result<(), AppError> {
        validate_post_fields(locale, &self.title, &self.slug, &self.content_md)
    }
}

/// 博文列表可选排序字段。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PostSortField {
    /// 按创建时间排序。
    CreatedAt,
    /// 按更新时间排序。
    UpdatedAt,
    /// 按标题排序。
    Title,
    /// 按 slug 排序。
    Slug,
}

impl PostSortField {
    /// 将查询参数中的排序字段解析为受限枚举。
    pub fn parse(value: Option<&str>) -> Result<Self, &'static str> {
        match value.map(str::trim).filter(|item| !item.is_empty()) {
            None => Ok(Self::CreatedAt),
            Some("created_at") => Ok(Self::CreatedAt),
            Some("updated_at") => Ok(Self::UpdatedAt),
            Some("title") => Ok(Self::Title),
            Some("slug") => Ok(Self::Slug),
            Some(_) => Err("sort"),
        }
    }
}

/// 博文列表归一化查询参数。
#[derive(Debug, Clone)]
pub struct NormalizedPostListQuery {
    /// 页码。
    pub page: i64,
    /// 每页条数。
    pub page_size: i64,
    /// 排序字段。
    pub sort: PostSortField,
    /// 排序方向。
    pub order: SortOrder,
    /// 过滤关键词。
    pub filter: Option<String>,
    /// 是否按发布状态过滤。
    pub published: Option<bool>,
}

/// 博文列表查询参数。
#[derive(Debug, Clone, Deserialize)]
pub struct PostListQuery {
    /// 页码。
    pub page: Option<i64>,
    /// 每页条数。
    pub page_size: Option<i64>,
    /// 排序字段。
    pub sort: Option<String>,
    /// 排序方向。
    pub order: Option<SortOrder>,
    /// 按标题或 slug 模糊搜索。
    pub filter: Option<String>,
    /// 按发布状态过滤。
    pub published: Option<bool>,
}

impl PostListQuery {
    /// 将原始查询参数校验并归一化为业务可执行的查询结构。
    pub fn normalize(
        self,
        locale: Locale,
        default_page: i64,
        default_page_size: i64,
        max_page_size: i64,
    ) -> Result<NormalizedPostListQuery, AppError> {
        let sort = PostSortField::parse(self.sort.as_deref()).map_err(|field| {
            AppError::BadRequestWithDetails(
                ErrorCode::InvalidParam,
                if matches!(locale, Locale::ZhCn) {
                    "sort 参数不合法".to_string()
                } else {
                    "invalid sort parameter".to_string()
                },
                vec![ValidationDetail::new(
                    field,
                    ValidationReason::InvalidFormat,
                )],
            )
        })?;

        Ok(NormalizedPostListQuery {
            page: self.page.unwrap_or(default_page).max(1),
            page_size: self
                .page_size
                .unwrap_or(default_page_size)
                .clamp(1, max_page_size),
            sort,
            order: self.order.unwrap_or(SortOrder::Desc),
            filter: self
                .filter
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty()),
            published: self.published,
        })
    }
}

/// 博文列表响应。
pub type PostListResponse = PageResponse<PostResponse>;

fn validate_post_fields(
    locale: Locale,
    title: &str,
    slug: &str,
    content_md: &str,
) -> Result<(), AppError> {
    if title.trim().is_empty() {
        return Err(AppError::BadRequestWithDetails(
            ErrorCode::InvalidParam,
            if matches!(locale, Locale::ZhCn) {
                "title 不能为空".to_string()
            } else {
                "title cannot be empty".to_string()
            },
            vec![ValidationDetail::new("title", ValidationReason::Required)],
        ));
    }

    if slug.trim().is_empty() {
        return Err(AppError::BadRequestWithDetails(
            ErrorCode::InvalidParam,
            if matches!(locale, Locale::ZhCn) {
                "slug 不能为空".to_string()
            } else {
                "slug cannot be empty".to_string()
            },
            vec![ValidationDetail::new("slug", ValidationReason::Required)],
        ));
    }

    if content_md.trim().is_empty() {
        return Err(AppError::BadRequestWithDetails(
            ErrorCode::InvalidParam,
            if matches!(locale, Locale::ZhCn) {
                "content_md 不能为空".to_string()
            } else {
                "content_md cannot be empty".to_string()
            },
            vec![ValidationDetail::new(
                "content_md",
                ValidationReason::Required,
            )],
        ));
    }

    Ok(())
}
