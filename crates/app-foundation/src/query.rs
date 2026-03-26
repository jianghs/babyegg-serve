use serde::{Deserialize, Serialize};

/// 通用排序方向。
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SortOrder {
    /// 升序排列。
    Asc,
    /// 降序排列。
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
    /// 页码，从 1 开始；为空时由 `normalize` 填充默认值。
    pub page: Option<i64>,
    /// 每页条数；为空时由 `normalize` 填充默认值。
    pub page_size: Option<i64>,
    /// 排序字段名，具体可选值由业务接口自行约束。
    pub sort: Option<String>,
    /// 排序方向。
    pub order: Option<SortOrder>,
    /// 预留过滤表达式。
    pub filter: Option<String>,
}

/// 归一化后的分页参数。
///
/// 该结构保证页码和页容量已经过默认值填充与边界裁剪，
/// 适合直接传入仓储层或分页计算逻辑。
#[derive(Debug, Clone, Copy)]
pub struct NormalizedListQuery {
    /// 归一化后的页码。
    pub page: i64,
    /// 归一化后的每页条数。
    pub page_size: i64,
}

impl ListQuery {
    /// 将原始查询参数归一化为可直接执行分页查询的结构。
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
