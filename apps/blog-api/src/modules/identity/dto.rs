use app_foundation::i18n::{translate, MessageKey};
use app_foundation::{AppError, ErrorCode, Locale, ValidationDetail, ValidationReason};

use crate::modules::{auth::dto::RegisterRequest, user::dto::CreateUserRequest};

#[derive(Debug, Clone)]
pub struct CreateIdentityUser {
    pub name: String,
    pub email: String,
    pub password: String,
}

impl CreateIdentityUser {
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
    fn from(value: RegisterRequest) -> Self {
        Self {
            name: value.name,
            email: value.email,
            password: value.password,
        }
    }
}

impl From<CreateUserRequest> for CreateIdentityUser {
    fn from(value: CreateUserRequest) -> Self {
        Self {
            name: value.name,
            email: value.email,
            password: value.password,
        }
    }
}
