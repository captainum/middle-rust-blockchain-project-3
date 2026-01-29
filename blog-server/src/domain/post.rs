//! Доменные модели поста.

use sqlx::types::chrono::{DateTime, Utc};
use serde::{Deserialize};

/// Информация о посте.
#[derive(Debug)]
struct Post {
    /// Идентификатор поста.
    pub id: i64,

    /// Заголовок поста.
    pub title: String,

    /// Содержимое поста.
    pub content: String,

    /// Идентификатор пользователя-автора поста.
    pub author_id: String,

    /// Время создания поста.
    pub created_at: DateTime<Utc>,

    /// Время последнего обновления поста.
    pub updated_at: DateTime<Utc>,
}

/// Данные о запросе на создание нового поста.
#[derive(Debug, Deserialize)]
struct CreatePostRequest {
    /// Заголовок поста.
    pub title: String,

    /// Содержимое поста.
    pub content: String,
}

/// Данные о запросе на обновление поста.
#[derive(Debug, Deserialize)]
struct UpdatePostRequest {
    /// Заголовок поста.
    pub title: Option<String>,

    /// Содержимое поста.
    pub content: Option<String>,
}