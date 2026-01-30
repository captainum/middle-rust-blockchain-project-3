//! Доменные модели поста.

use sqlx::types::chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Информация о посте.
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Post {
    /// Идентификатор поста.
    pub id: i64,

    /// Заголовок поста.
    pub title: String,

    /// Содержимое поста.
    pub content: String,

    /// Идентификатор пользователя-автора поста.
    pub author_id: i64,

    /// Время создания поста.
    pub created_at: DateTime<Utc>,

    /// Время последнего обновления поста.
    pub updated_at: DateTime<Utc>,
}

crate::impl_json_response!(Post);

/// Данные о запросе на создание нового поста.
#[derive(Debug, Deserialize)]
pub struct CreatePostRequest {
    /// Заголовок поста.
    pub title: String,

    /// Содержимое поста.
    pub content: String,
}

impl From<CreatePostRequest> for Post {
    fn from(post: CreatePostRequest) -> Self {
        Self {
            id: -1,
            title: post.title,
            content: post.content,
            author_id: -1,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

/// Данные о запросе на обновление поста.
#[derive(Debug, Deserialize)]
pub struct UpdatePostRequest {
    /// Идентификатор поста.
    #[serde(skip)]
    pub id: i64,

    /// Заголовок поста.
    pub title: Option<String>,

    /// Содержимое поста.
    pub content: Option<String>,
}