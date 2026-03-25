use std::fmt::{Display, Formatter};

/// 稳定的业务错误码。
///
/// - 用于客户端逻辑判断，不依赖自然语言文案
/// - 不随国际化语言变化
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    InvalidParam,
    Forbidden,
    NotFound,
    InternalError,
    AuthInvalidCredentials,
    AuthMissingAuthorizationHeader,
    AuthInvalidAuthorizationHeader,
    AuthInvalidToken,
    AuthInvalidTokenSubject,
    AuthForbiddenRole,
    AuthForbiddenScope,
    UserNameEmpty,
    UserEmailEmpty,
    UserPasswordEmpty,
    UserPasswordTooShort,
    UserEmailExists,
}

impl ErrorCode {
    pub fn as_str(self) -> &'static str {
        match self {
            ErrorCode::InvalidParam => "INVALID_PARAM",
            ErrorCode::Forbidden => "FORBIDDEN",
            ErrorCode::NotFound => "NOT_FOUND",
            ErrorCode::InternalError => "INTERNAL_ERROR",
            ErrorCode::AuthInvalidCredentials => "AUTH_INVALID_CREDENTIALS",
            ErrorCode::AuthMissingAuthorizationHeader => "AUTH_MISSING_AUTHORIZATION_HEADER",
            ErrorCode::AuthInvalidAuthorizationHeader => "AUTH_INVALID_AUTHORIZATION_HEADER",
            ErrorCode::AuthInvalidToken => "AUTH_INVALID_TOKEN",
            ErrorCode::AuthInvalidTokenSubject => "AUTH_INVALID_TOKEN_SUBJECT",
            ErrorCode::AuthForbiddenRole => "AUTH_FORBIDDEN_ROLE",
            ErrorCode::AuthForbiddenScope => "AUTH_FORBIDDEN_SCOPE",
            ErrorCode::UserNameEmpty => "USER_NAME_EMPTY",
            ErrorCode::UserEmailEmpty => "USER_EMAIL_EMPTY",
            ErrorCode::UserPasswordEmpty => "USER_PASSWORD_EMPTY",
            ErrorCode::UserPasswordTooShort => "USER_PASSWORD_TOO_SHORT",
            ErrorCode::UserEmailExists => "USER_EMAIL_EXISTS",
        }
    }
}

impl Display for ErrorCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}
