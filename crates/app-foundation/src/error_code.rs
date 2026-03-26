use std::fmt::{Display, Formatter};

/// 稳定的业务错误码。
///
/// - 用于客户端逻辑判断，不依赖自然语言文案
/// - 不随国际化语言变化
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    /// 通用参数错误。
    InvalidParam,
    /// 通用权限不足。
    Forbidden,
    /// 资源不存在。
    NotFound,
    /// 通用内部错误。
    InternalError,
    /// 登录凭证无效。
    AuthInvalidCredentials,
    /// 缺少鉴权请求头。
    AuthMissingAuthorizationHeader,
    /// 鉴权请求头格式不合法。
    AuthInvalidAuthorizationHeader,
    /// 访问令牌无效。
    AuthInvalidToken,
    /// 访问令牌主题字段不合法。
    AuthInvalidTokenSubject,
    /// 缺少刷新令牌。
    AuthMissingRefreshToken,
    /// 刷新令牌无效。
    AuthInvalidRefreshToken,
    /// 缺少所需角色。
    AuthForbiddenRole,
    /// 缺少所需权限范围。
    AuthForbiddenScope,
    /// 用户名为空。
    UserNameEmpty,
    /// 用户邮箱为空。
    UserEmailEmpty,
    /// 用户密码为空。
    UserPasswordEmpty,
    /// 用户密码长度不足。
    UserPasswordTooShort,
    /// 用户邮箱已存在。
    UserEmailExists,
}

impl ErrorCode {
    /// 返回稳定的错误码字符串。
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
            ErrorCode::AuthMissingRefreshToken => "AUTH_MISSING_REFRESH_TOKEN",
            ErrorCode::AuthInvalidRefreshToken => "AUTH_INVALID_REFRESH_TOKEN",
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
