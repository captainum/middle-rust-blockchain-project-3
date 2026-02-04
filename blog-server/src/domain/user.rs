//! Доменные модели пользователя.

use crate::domain::error::UserError;
use crate::impl_json_response;
use argon2::{
    Argon2, PasswordHasher,
    password_hash::{SaltString, rand_core::OsRng},
};
use serde::{Deserialize, Serialize};
use sqlx::types::chrono::{DateTime, Utc};

use validator::Validate;

/// Информация о пользователе.
#[derive(Debug, Serialize)]
pub struct User {
    /// Идентификатор пользователя.
    pub id: i64,

    /// Имя пользователя.
    pub username: String,

    /// Email-адрес пользователя.
    pub email: String,

    #[serde(skip)]
    /// Хеш от пароля пользователя.
    pub password_hash: String,

    /// Время создания пользователя.
    pub created_at: DateTime<Utc>,
}

impl From<User> for crate::blog_grpc::User {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
            created_at: user.created_at.to_string(),
        }
    }
}

/// Данные о запросе на создание нового пользователя.
#[derive(Debug, Deserialize, Validate)]
pub struct CreateUserRequest {
    /// Имя пользователя.
    pub username: String,

    /// Email-адрес пользователя.
    #[validate(email)]
    pub email: String,

    /// Пароль пользователя.
    #[validate(length(min = 6))]
    pub password: String,
}

impl From<crate::blog_grpc::CreateUserRequest> for CreateUserRequest {
    fn from(req: crate::blog_grpc::CreateUserRequest) -> Self {
        Self {
            username: req.username,
            email: req.email,
            password: req.password,
        }
    }
}

impl TryFrom<CreateUserRequest> for User {
    type Error = UserError;

    fn try_from(user: CreateUserRequest) -> Result<Self, Self::Error> {
        let password_hash = Argon2::default()
            .hash_password(user.password.as_bytes(), &SaltString::generate(&mut OsRng))?
            .to_string();

        Ok(Self {
            id: -1,
            username: user.username,
            email: user.email,
            password_hash,
            created_at: Utc::now(),
        })
    }
}

/// Данные об ответе на создание нового пользователя.
#[derive(Debug, Serialize)]
pub struct CreateUserResponse {
    /// JWT-токен авторизации.
    pub token: String,

    /// Созданный пользователь.
    pub user: User,
}

impl_json_response!(CreateUserResponse);

impl From<CreateUserResponse> for crate::blog_grpc::CreateUserResponse {
    fn from(response: CreateUserResponse) -> Self {
        Self {
            token: response.token,
            user: Some(response.user.into()),
        }
    }
}

/// Данные о запросе на вход пользователя.
#[derive(Debug, Deserialize)]
pub(crate) struct LoginUserRequest {
    /// Имя пользователя.
    pub username: String,

    /// Пароль пользователя.
    pub password: String,
}

impl From<crate::blog_grpc::LoginUserRequest> for LoginUserRequest {
    fn from(req: crate::blog_grpc::LoginUserRequest) -> Self {
        Self {
            username: req.username,
            password: req.password,
        }
    }
}

/// Данные об ответе на вход пользователя.
#[derive(Debug, Serialize)]
pub struct LoginUserResponse {
    /// JWT-токен авторизации.
    pub token: String,

    /// Информация о пользователе, который был авторизован.
    pub user: User,
}

impl_json_response!(LoginUserResponse);

impl From<LoginUserResponse> for crate::blog_grpc::LoginUserResponse {
    fn from(response: LoginUserResponse) -> Self {
        Self {
            token: response.token,
            user: Some(response.user.into()),
        }
    }
}
