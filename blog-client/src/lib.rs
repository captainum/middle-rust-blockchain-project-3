use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::net::SocketAddr;
use tonic::{async_trait, Request};

pub mod error;
mod grpc_client;
mod http_client;

use error::BlogClientError;

mod blog_grpc {
    tonic::include_proto!("blog");
}

use crate::grpc_client::GrpcClient;
use crate::http_client::HttpClient;

#[derive(Debug, Deserialize)]
pub struct AuthResponse {
    pub token: String,
}

/// Протокол для взаимодействия.
pub enum Transport {
    Http(SocketAddr),
    Grpc(SocketAddr),
}

/// Клиент для взаимодействия с серверной частью системы блога.
pub struct BlogClient {
    inner: Box<dyn Client<Error = BlogClientError>>,
    token: Option<String>,
}

#[async_trait]
pub trait Client {
    type Error;

    async fn register(
        &mut self,
        username: &str,
        email: &str,
        password: &str,
    ) -> Result<String, Self::Error>;

    async fn login(&mut self, username: &str, password: &str) -> Result<String, Self::Error>;

    async fn create_post(&mut self, token: &str, title: &str, content: &str) -> Result<Post, Self::Error>;

    async fn get_post(&mut self, id: i64) -> Result<Post, Self::Error>;

    async fn get_posts(&mut self, limit: i64, offset: i64) -> Result<Vec<Post>, Self::Error>;

    async fn update_post(&mut self, token: &str, id: i64, title: Option<String>, content: Option<String>) -> Result<Post, Self::Error>;

    async fn delete_post(&mut self, token: &str, id: i64) -> Result<(), Self::Error>;
}

impl BlogClient {
    pub async fn new(transport: Transport) -> anyhow::Result<Self> {
        let client = match transport {
            Transport::Http(addr) => Self {
                inner: Box::new(HttpClient::new(addr).await?),
                token: None,
            },
            Transport::Grpc(addr) => Self {
                inner: Box::new(GrpcClient::new(addr).await?),
                token: None,
            },
        };

        Ok(client)
    }

    pub fn set_token(&mut self, token: String) {
        self.token = Some(token);
    }

    pub fn get_token(&self) -> Option<String> {
        self.token.clone()
    }

    pub async fn register(
        &mut self,
        username: &str,
        email: &str,
        password: &str,
    ) -> Result<(), BlogClientError> {
        let token = self.inner.register(username, email, password).await?;
        self.set_token(token);

        Ok(())
    }

    pub async fn login(&mut self, username: &str, password: &str) -> Result<(), BlogClientError> {
        let token = self.inner.login(username, password).await?;

        self.set_token(token);

        Ok(())
    }

    pub async fn create_post(&mut self, title: &str, content: &str) -> Result<Post, BlogClientError> {
        let token = self.get_token().ok_or(BlogClientError::TokenNotFound)?;

        let post = self.inner.create_post(&token, title, content).await?;

        Ok(post)
    }

    pub async fn get_post(&mut self, id: i64) -> Result<Post, BlogClientError> {
        let post = self.inner.get_post(id).await?;

        Ok(post)
    }

    pub async fn get_posts(&mut self, limit: i64, offset: i64) -> Result<Vec<Post>, BlogClientError> {
        let posts = self.inner.get_posts(limit, offset).await?;

        Ok(posts)
    }

    pub async fn update_post(&mut self, id: i64, title: Option<String>, content: Option<String>) -> Result<Post, BlogClientError> {
        let token = self.get_token().ok_or(BlogClientError::TokenNotFound)?;

        let post = self.inner.update_post(&token, id, title, content).await?;

        Ok(post)
    }

    pub async fn delete_post(&mut self, id: i64) -> Result<(), BlogClientError> {
        let token = self.get_token().ok_or(BlogClientError::TokenNotFound)?;

        self.inner.delete_post(&token, id).await?;

        Ok(())
    }
}

/// Информация о пользователе.
#[derive(Debug, Deserialize)]
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

/// Информация о посте.
#[derive(Debug, Deserialize)]
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

impl TryFrom<blog_grpc::Post> for Post {
    type Error = BlogClientError;

    fn try_from(post: blog_grpc::Post) -> Result<Self, Self::Error> {
        Ok(Self {
            id: post.id,
            title: post.title,
            content: post.content,
            author_id: post.author_id,
            created_at: post.created_at.parse().map_err(|_| BlogClientError::InvalidPostContent)?,
            updated_at: post.updated_at.parse().map_err(|_| BlogClientError::InvalidPostContent)?,
        })
    }
}
