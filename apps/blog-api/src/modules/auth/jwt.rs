use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

/// JWT Claims。
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

pub fn create_token(
    user_id: Uuid,
    secret: &str,
    expire_seconds: i64,
) -> Result<String, jsonwebtoken::errors::Error> {
    let now = OffsetDateTime::now_utc().unix_timestamp();

    let claims = Claims {
        sub: user_id.to_string(),
        iat: now as usize,
        exp: (now + expire_seconds) as usize,
        roles: vec!["user".to_string()],
        scopes: vec!["*".to_string()],
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
}

pub fn verify_token(token: &str, secret: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
}
