use serde::Serialize;

/// 通用接口成功响应。
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    /// 业务状态码，0 表示成功
    pub code: i32,
    /// 提示信息
    pub message: String,
    /// 实际返回数据
    pub data: T,
}

impl<T> ApiResponse<T> {
    pub fn ok(data: T) -> Self {
        Self {
            code: 0,
            message: "ok".to_string(),
            data,
        }
    }
}

/// 通用分页响应结构。
///
/// 适用于用户列表、文章列表、评论列表等所有分页接口。
#[derive(Debug, Serialize)]
pub struct PageResponse<T> {
    /// 当前页数据列表
    pub items: Vec<T>,
    /// 当前页码，从 1 开始
    pub page: i64,
    /// 每页条数
    pub page_size: i64,
    /// 总记录数
    pub total: i64,
    /// 总页数
    pub total_pages: i64,
}

impl<T> PageResponse<T> {
    /// 构造分页响应。
    pub fn new(items: Vec<T>, page: i64, page_size: i64, total: i64) -> Self {
        let total_pages = if total == 0 {
            0
        } else {
            (total + page_size - 1) / page_size
        };

        Self {
            items,
            page,
            page_size,
            total,
            total_pages,
        }
    }
}
