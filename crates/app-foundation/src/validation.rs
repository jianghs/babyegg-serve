use serde::Serialize;

/// 参数校验错误明细。
#[derive(Debug, Clone, Serialize)]
pub struct ValidationDetail {
    pub field: String,
    pub reason: String,
}

impl ValidationDetail {
    pub fn new(field: &str, reason: &str) -> Self {
        Self {
            field: field.to_string(),
            reason: reason.to_string(),
        }
    }
}
