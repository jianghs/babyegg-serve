use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use thiserror::Error;

use crate::error_code::ErrorCode;
use crate::validation::ValidationDetail;

/// 通用应用错误。
/// 业务项目可在自己的 crate 中继续扩展，或者包装成更细的错误类型。
#[derive(Debug, Error)]
pub enum AppError {
    /// 返回 400，消息来自调用方提供的自由文本。
    #[error("{0}")]
    BadRequest(String),
    /// 返回 400，并附带机器可读错误码。
    #[error("{1}")]
    BadRequestWithCode(ErrorCode, String),
    /// 返回 400，并附带字段级校验详情。
    #[error("{1}")]
    BadRequestWithDetails(ErrorCode, String, Vec<ValidationDetail>),
    /// 返回 403，并附带机器可读错误码。
    #[error("{1}")]
    ForbiddenWithCode(ErrorCode, String),

    /// 返回 404，使用默认消息。
    #[error("not found")]
    NotFound,
    /// 返回 404，并附带自定义消息。
    #[error("{0}")]
    NotFoundWithMessage(String),
    /// 返回 404，并附带机器可读错误码。
    #[error("{1}")]
    NotFoundWithCode(ErrorCode, String),

    /// 返回 500，使用默认消息。
    #[error("internal server error")]
    Internal,
    /// 返回 500，并附带自定义消息。
    #[error("{0}")]
    InternalWithMessage(String),
    /// 返回 500，并附带机器可读错误码。
    #[error("{1}")]
    InternalWithCode(ErrorCode, String),
}

#[derive(Debug, Serialize)]
struct ErrorBody {
    code: i32,
    error_code: String,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<Vec<ValidationDetail>>,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let error_code = self.error_code();
        let details = self.validation_details().map(|d| d.to_vec());

        let body = Json(ErrorBody {
            code: status.as_u16() as i32,
            error_code: error_code.to_string(),
            message: self.to_string(),
            details,
        });

        (status, body).into_response()
    }
}

impl AppError {
    /// 将应用错误映射为对应的 HTTP 状态码。
    pub fn status_code(&self) -> StatusCode {
        match self {
            AppError::BadRequest(_)
            | AppError::BadRequestWithCode(_, _)
            | AppError::BadRequestWithDetails(_, _, _) => StatusCode::BAD_REQUEST,
            AppError::ForbiddenWithCode(_, _) => StatusCode::FORBIDDEN,
            AppError::NotFound
            | AppError::NotFoundWithMessage(_)
            | AppError::NotFoundWithCode(_, _) => StatusCode::NOT_FOUND,
            AppError::Internal
            | AppError::InternalWithMessage(_)
            | AppError::InternalWithCode(_, _) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// 返回错误对应的机器可读错误码。
    pub fn error_code(&self) -> ErrorCode {
        match self {
            AppError::BadRequest(_) => ErrorCode::InvalidParam,
            AppError::BadRequestWithCode(code, _) => *code,
            AppError::BadRequestWithDetails(code, _, _) => *code,
            AppError::ForbiddenWithCode(code, _) => *code,
            AppError::NotFound | AppError::NotFoundWithMessage(_) => ErrorCode::NotFound,
            AppError::NotFoundWithCode(code, _) => *code,
            AppError::Internal | AppError::InternalWithMessage(_) => ErrorCode::InternalError,
            AppError::InternalWithCode(code, _) => *code,
        }
    }

    /// 返回字段级校验详情；非校验类错误时为空。
    pub fn validation_details(&self) -> Option<&[ValidationDetail]> {
        match self {
            AppError::BadRequestWithDetails(_, _, details) => Some(details.as_slice()),
            _ => None,
        }
    }
}
