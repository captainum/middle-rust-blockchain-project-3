//! Доменные модели пользователя.

use sqlx::types::chrono::{DateTime, Utc};
use serde::{Deserialize};

/// Информация о пользователе.
#[derive(Debug)]
struct User {
    /// Идентификатор пользователя.
    id: i64,

    /// Имя пользователя.
    username: String,

    /// Email-адрес пользователя.
    email: String,

    /// Хеш от пароля пользователя.
    password_hash: String,

    /// Время создания пользователя.
    created_at: DateTime<Utc>,
}

/// Данные о запросе на создание нового пользователя.
#[derive(Debug, Deserialize)]
struct CreateUserRequest {
    /// Имя пользователя.
    username: String,

    /// Email-адрес пользователя.
    email: String,

    /// Пароль пользователя.
    password: String,
}

/// Данные о запросе на вход пользователя.
#[derive(Debug, Deserialize)]
struct LoginUserRequest {
    /// Имя пользователя.
    username: String,

    /// Пароль пользователя.
    password: String,
}
