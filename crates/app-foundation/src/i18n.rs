use crate::locale::Locale;

/// 基础框架层的消息 key。
/// 只放公共基础设施日志，不放业务特定文案。
#[derive(Debug, Clone, Copy)]
pub enum MessageKey {
    /// 服务启动日志。
    ServerStarted,
    /// 服务停止日志。
    ServerStopped,
    /// 收到请求日志。
    RequestReceived,
    /// 通用 500 文案。
    InternalServerError,
    /// 姓名为空。
    NameCannotBeEmpty,
    /// 邮箱为空。
    EmailCannotBeEmpty,
    /// 密码为空。
    PasswordCannotBeEmpty,
    /// 密码过短。
    PasswordTooShort,
    /// 邮箱已存在。
    EmailAlreadyExists,
    /// 登录凭证无效。
    InvalidEmailOrPassword,
    /// 缺少鉴权请求头。
    MissingAuthorizationHeader,
    /// 鉴权请求头格式错误。
    InvalidAuthorizationHeader,
    /// 访问令牌无效。
    InvalidToken,
    /// 访问令牌主题无效。
    InvalidTokenSubject,
    /// 缺少刷新令牌。
    MissingRefreshToken,
    /// 刷新令牌无效。
    InvalidRefreshToken,
    /// 角色权限不足。
    ForbiddenRole,
    /// Scope 权限不足。
    ForbiddenScope,
    /// 资源不存在。
    NotFound,
}

/// 根据语言区域和消息键返回对应文案。
pub fn translate(locale: Locale, key: MessageKey) -> &'static str {
    match locale {
        Locale::ZhCn => match key {
            MessageKey::ServerStarted => "服务已启动",
            MessageKey::ServerStopped => "服务已停止",
            MessageKey::RequestReceived => "收到请求",
            MessageKey::InternalServerError => "内部服务器错误",
            MessageKey::NameCannotBeEmpty => "姓名不能为空",
            MessageKey::EmailCannotBeEmpty => "邮箱不能为空",
            MessageKey::PasswordCannotBeEmpty => "密码不能为空",
            MessageKey::PasswordTooShort => "密码长度至少为 6 位",
            MessageKey::EmailAlreadyExists => "邮箱已存在",
            MessageKey::InvalidEmailOrPassword => "邮箱或密码错误",
            MessageKey::MissingAuthorizationHeader => "缺少 authorization 请求头",
            MessageKey::InvalidAuthorizationHeader => "authorization 请求头格式错误",
            MessageKey::InvalidToken => "无效 token",
            MessageKey::InvalidTokenSubject => "无效 token subject",
            MessageKey::MissingRefreshToken => "缺少 refresh token",
            MessageKey::InvalidRefreshToken => "无效 refresh token",
            MessageKey::ForbiddenRole => "没有访问该角色资源的权限",
            MessageKey::ForbiddenScope => "没有访问该范围资源的权限",
            MessageKey::NotFound => "资源不存在",
        },
        Locale::EnUs => match key {
            MessageKey::ServerStarted => "Server started",
            MessageKey::ServerStopped => "Server stopped",
            MessageKey::RequestReceived => "Request received",
            MessageKey::InternalServerError => "Internal server error",
            MessageKey::NameCannotBeEmpty => "name cannot be empty",
            MessageKey::EmailCannotBeEmpty => "email cannot be empty",
            MessageKey::PasswordCannotBeEmpty => "password cannot be empty",
            MessageKey::PasswordTooShort => "password must be at least 6 characters",
            MessageKey::EmailAlreadyExists => "email already exists",
            MessageKey::InvalidEmailOrPassword => "invalid email or password",
            MessageKey::MissingAuthorizationHeader => "missing authorization header",
            MessageKey::InvalidAuthorizationHeader => "invalid authorization header",
            MessageKey::InvalidToken => "invalid token",
            MessageKey::InvalidTokenSubject => "invalid token subject",
            MessageKey::MissingRefreshToken => "missing refresh token",
            MessageKey::InvalidRefreshToken => "invalid refresh token",
            MessageKey::ForbiddenRole => "forbidden role",
            MessageKey::ForbiddenScope => "forbidden scope",
            MessageKey::NotFound => "not found",
        },
    }
}
