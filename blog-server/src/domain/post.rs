//! Доменные модели поста.

use serde::{Deserialize, Serialize};
use sqlx::types::chrono::{DateTime, Utc};

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

impl From<Post> for crate::blog_grpc::Post {
    fn from(post: Post) -> Self {
        Self {
            id: post.id,
            title: post.title,
            content: post.content,
            author_id: post.author_id,
            created_at: post.created_at.to_rfc3339(),
            updated_at: post.updated_at.to_rfc3339(),
        }
    }
}

/// Данные о запросе на создание нового поста.
#[derive(Debug, Deserialize)]
pub struct CreatePostRequest {
    /// Заголовок поста.
    pub title: String,

    /// Содержимое поста.
    pub content: String,
}

impl From<crate::blog_grpc::CreatePostRequest> for CreatePostRequest {
    fn from(req: crate::blog_grpc::CreatePostRequest) -> Self {
        Self {
            title: req.title,
            content: req.content,
        }
    }
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
pub(crate) struct UpdatePostRequest {
    /// Идентификатор поста.
    #[serde(skip)]
    pub id: i64,

    /// Заголовок поста.
    pub title: Option<String>,

    /// Содержимое поста.
    pub content: Option<String>,
}

impl From<crate::blog_grpc::UpdatePostRequest> for UpdatePostRequest {
    fn from(req: crate::blog_grpc::UpdatePostRequest) -> Self {
        Self {
            id: req.id,
            title: req.title,
            content: req.content,
        }
    }
}
