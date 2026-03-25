use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use thiserror::Error;

/// 通用应用错误。
/// 业务项目可在自己的 crate 中继续扩展，或者包装成更细的错误类型。
#[derive(Debug, Error)]
pub enum AppError {
    #[error("{0}")]
    BadRequest(String),

    #[error("not found")]
    NotFound,
    #[error("{0}")]
    NotFoundWithMessage(String),

    #[error("internal server error")]
    Internal,
    #[error("{0}")]
    InternalWithMessage(String),
}

#[derive(Debug, Serialize)]
struct ErrorBody {
    code: i32,
    message: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match self {
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::NotFound | AppError::NotFoundWithMessage(_) => StatusCode::NOT_FOUND,
            AppError::Internal | AppError::InternalWithMessage(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        };

        let body = Json(ErrorBody {
            code: status.as_u16() as i32,
            message: self.to_string(),
        });

        (status, body).into_response()
    }
}
