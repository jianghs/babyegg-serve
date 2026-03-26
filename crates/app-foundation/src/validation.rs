use serde::Serialize;

/// 参数校验错误明细。
#[derive(Debug, Clone, Serialize)]
pub struct ValidationDetail {
    /// 校验失败的字段名。
    pub field: String,
    /// 校验失败原因。
    pub reason: ValidationReason,
}

/// 参数校验失败原因（机器可读）。
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationReason {
    /// 必填字段缺失。
    Required,
    /// 格式非法。
    InvalidFormat,
    /// 最小长度 6。
    #[serde(rename = "min_length_6")]
    MinLength6,
}

impl ValidationDetail {
    /// 构造一条字段级校验详情。
    pub fn new(field: &str, reason: ValidationReason) -> Self {
        Self {
            field: field.to_string(),
            reason,
        }
    }
}
