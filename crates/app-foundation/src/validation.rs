use serde::Serialize;

/// 参数校验错误明细。
#[derive(Debug, Clone, Serialize)]
pub struct ValidationDetail {
    pub field: String,
    pub reason: ValidationReason,
}

/// 参数校验失败原因（机器可读）。
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationReason {
    Required,
    InvalidFormat,
    #[serde(rename = "min_length_6")]
    MinLength6,
}

impl ValidationDetail {
    pub fn new(field: &str, reason: ValidationReason) -> Self {
        Self {
            field: field.to_string(),
            reason,
        }
    }
}
