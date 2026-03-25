use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use thiserror::Error;

use crate::error_code::ErrorCode;

/// 通用应用错误。
/// 业务项目可在自己的 crate 中继续扩展，或者包装成更细的错误类型。
#[derive(Debug, Error)]
pub enum AppError {
    #[error("{0}")]
    BadRequest(String),
    #[error("{1}")]
    BadRequestWithCode(ErrorCode, String),

    #[error("not found")]
    NotFound,
    #[error("{0}")]
    NotFoundWithMessage(String),
    #[error("{1}")]
    NotFoundWithCode(ErrorCode, String),

    #[error("internal server error")]
    Internal,
    #[error("{0}")]
    InternalWithMessage(String),
    #[error("{1}")]
    InternalWithCode(ErrorCode, String),
}

#[derive(Debug, Serialize)]
struct ErrorBody {
    code: i32,
    error_code: String,
    message: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let error_code = self.error_code();

        let body = Json(ErrorBody {
            code: status.as_u16() as i32,
            error_code: error_code.to_string(),
            message: self.to_string(),
        });

        (status, body).into_response()
    }
}

impl AppError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            AppError::BadRequest(_) | AppError::BadRequestWithCode(_, _) => StatusCode::BAD_REQUEST,
            AppError::NotFound
            | AppError::NotFoundWithMessage(_)
            | AppError::NotFoundWithCode(_, _) => StatusCode::NOT_FOUND,
            AppError::Internal
            | AppError::InternalWithMessage(_)
            | AppError::InternalWithCode(_, _) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub fn error_code(&self) -> ErrorCode {
        match self {
            AppError::BadRequest(_) => ErrorCode::InvalidParam,
            AppError::BadRequestWithCode(code, _) => *code,
            AppError::NotFound | AppError::NotFoundWithMessage(_) => ErrorCode::NotFound,
            AppError::NotFoundWithCode(code, _) => *code,
            AppError::Internal | AppError::InternalWithMessage(_) => ErrorCode::InternalError,
            AppError::InternalWithCode(code, _) => *code,
        }
    }
}
