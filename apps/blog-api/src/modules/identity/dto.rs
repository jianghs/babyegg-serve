use app_foundation::i18n::{translate, MessageKey};
use app_foundation::{AppError, ErrorCode, Locale, ValidationDetail, ValidationReason};

use crate::modules::{auth::dto::RegisterRequest, user::dto::CreateUserRequest};

/// 身份域内部使用的建用户输入模型。
///
/// 该结构统一承接注册接口与后台创建用户接口的输入，
/// 以便复用相同的字段校验和建档逻辑。
#[derive(Debug, Clone)]
pub struct CreateIdentityUser {
    /// 用户显示名称。
    pub name: String,
    /// 登录邮箱。
    pub email: String,
    /// 明文密码。
    pub password: String,
}

impl CreateIdentityUser {
    /// 校验身份域创建用户输入。
    pub fn validate(&self, locale: Locale) -> Result<(), AppError> {
        if self.name.trim().is_empty() {
            return Err(AppError::BadRequestWithDetails(
                ErrorCode::UserNameEmpty,
                translate(locale, MessageKey::NameCannotBeEmpty).to_string(),
                vec![ValidationDetail::new("name", ValidationReason::Required)],
            ));
        }

        if self.email.trim().is_empty() {
            return Err(AppError::BadRequestWithDetails(
                ErrorCode::UserEmailEmpty,
                translate(locale, MessageKey::EmailCannotBeEmpty).to_string(),
                vec![ValidationDetail::new("email", ValidationReason::Required)],
            ));
        }

        if self.password.len() < 6 {
            return Err(AppError::BadRequestWithDetails(
                ErrorCode::UserPasswordTooShort,
                translate(locale, MessageKey::PasswordTooShort).to_string(),
                vec![ValidationDetail::new(
                    "password",
                    ValidationReason::MinLength6,
                )],
            ));
        }

        Ok(())
    }
}

impl From<RegisterRequest> for CreateIdentityUser {
    /// 将注册请求转换为身份域统一输入模型。
    fn from(value: RegisterRequest) -> Self {
        Self {
            name: value.name,
            email: value.email,
            password: value.password,
        }
    }
}

impl From<CreateUserRequest> for CreateIdentityUser {
    /// 将用户管理接口的创建请求转换为身份域统一输入模型。
    fn from(value: CreateUserRequest) -> Self {
        Self {
            name: value.name,
            email: value.email,
            password: value.password,
        }
    }
}
