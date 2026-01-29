//! Доменные модели пользователя.

use sqlx::types::chrono::{DateTime, Utc};
use serde::{Deserialize};

/// Информация о пользователе.
#[derive(Debug)]
struct User {
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
struct CreateUserRequest {
    /// Имя пользователя.
    pub username: String,

    /// Email-адрес пользователя.
    pub email: String,

    /// Пароль пользователя.
    pub password: String,
}

/// Данные о запросе на вход пользователя.
#[derive(Debug, Deserialize)]
struct LoginUserRequest {
    /// Имя пользователя.
    pub username: String,

    /// Пароль пользователя.
    pub password: String,
}
