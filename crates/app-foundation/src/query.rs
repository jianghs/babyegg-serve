use serde::{Deserialize, Serialize};

/// 通用排序方向。
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SortOrder {
    Asc,
    Desc,
}

/// 通用列表查询参数。
///
/// 约定字段：
/// - page/page_size: 分页
/// - sort/order: 排序字段与方向
/// - filter: 预留通用过滤表达式（具体语义由业务定义）
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ListQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub sort: Option<String>,
    pub order: Option<SortOrder>,
    pub filter: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub struct NormalizedListQuery {
    pub page: i64,
    pub page_size: i64,
}

impl ListQuery {
    pub fn normalize(
        self,
        default_page: i64,
        default_page_size: i64,
        max_page_size: i64,
    ) -> NormalizedListQuery {
        let page = self.page.unwrap_or(default_page).max(1);
        let page_size = self
            .page_size
            .unwrap_or(default_page_size)
            .clamp(1, max_page_size);

        NormalizedListQuery { page, page_size }
    }
}
