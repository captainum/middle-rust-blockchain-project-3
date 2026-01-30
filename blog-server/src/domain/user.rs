//! Доменные модели пользователя.

use argon2::{password_hash::{SaltString, rand_core::OsRng}, Argon2, PasswordHasher};
use sqlx::types::chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::domain::error::UserError;
use crate::impl_json_response;

/// Информация о пользователе.
#[derive(Debug)]
pub struct User {
    /// Идентификатор пользователя.
    pub id: i64,

    /// Имя пользователя.
    pub username: String,

    /// Email-адрес пользователя.
    pub email: String,

    /// Хеш от пароля пользователя.
    pub password_hash: String,

    /// Время создания пользователя.
    pub created_at: DateTime<Utc>,
}

/// Данные о запросе на создание нового пользователя.
#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    /// Имя пользователя.
    pub username: String,

    /// Email-адрес пользователя.
    pub email: String,

    /// Пароль пользователя.
    pub password: String,
}

impl TryFrom<CreateUserRequest> for User {
    type Error = UserError;

    fn try_from(user: CreateUserRequest) -> Result<Self, Self::Error> {
        let password_hash = Argon2::default()
            .hash_password(user.password.as_bytes(), &SaltString::generate(&mut OsRng))?.to_string();

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
    pub token: String,
}

impl_json_response!(CreateUserResponse);

/// Данные о запросе на вход пользователя.
#[derive(Debug, Deserialize)]
pub struct LoginUserRequest {
    /// Имя пользователя.
    pub username: String,

    /// Пароль пользователя.
    pub password: String,
}

/// Данные об ответе на вход пользователя.
#[derive(Debug, Serialize)]
pub struct LoginUserResponse {
    pub token: String,
}

impl_json_response!(LoginUserResponse);
