use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

/// JWT Claims。
///
/// 当前访问令牌中的声明约定如下：
/// - `sub`：用户 ID 字符串
/// - `iat`：签发时间，Unix 时间戳秒级值
/// - `exp`：过期时间，Unix 时间戳秒级值
/// - `roles`：角色键集合
/// - `scopes`：权限键集合
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// 用户 ID
    pub sub: String,
    /// 签发时间
    pub iat: usize,
    /// 过期时间
    pub exp: usize,
    /// 角色列表
    #[serde(default)]
    pub roles: Vec<String>,
    /// 权限范围
    #[serde(default)]
    pub scopes: Vec<String>,
}

/// 为指定用户创建访问令牌。
///
/// 该函数只负责编码 access token，不负责 refresh token 持久化。
pub fn create_token(
    user_id: Uuid,
    secret: &str,
    expire_seconds: i64,
    roles: Vec<String>,
    scopes: Vec<String>,
) -> Result<String, jsonwebtoken::errors::Error> {
    let now = OffsetDateTime::now_utc().unix_timestamp();

    let claims = Claims {
        sub: user_id.to_string(),
        iat: now as usize,
        exp: (now + expire_seconds) as usize,
        roles,
        scopes,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
}

/// 校验访问令牌签名与过期时间，并解析出声明体。
///
/// 当前使用 `jsonwebtoken` 默认校验策略，会校验签名、过期时间等标准字段。
pub fn verify_token(token: &str, secret: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
}
