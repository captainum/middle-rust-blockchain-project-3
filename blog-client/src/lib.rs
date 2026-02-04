//! Библиотека клиента для системы блога.
//!
//! Предоставляет абстракцию для взаимодействия с серверной частью блога
//! через различные транспортные протоколы (HTTP, gRPC).

use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::net::SocketAddr;
use tonic::async_trait;

pub mod error;
mod grpc_client;
mod http_client;

use error::BlogClientError;

mod blog_grpc {
    tonic::include_proto!("blog");
}

use crate::grpc_client::GrpcClient;
use crate::http_client::HttpClient;

/// Ответ сервера с JWT-токеном при авторизации.
#[derive(Debug, Deserialize)]
pub struct AuthResponse {
    /// JWT-токен для последующих запросов.
    pub token: String,

    /// Авторизованный пользователь.
    pub user: User,
}

impl TryFrom<blog_grpc::CreateUserResponse> for AuthResponse {
    type Error = BlogClientError;

    fn try_from(response: blog_grpc::CreateUserResponse) -> Result<Self, Self::Error> {
        match response.user {
            Some(user) => Ok(Self {
                token: response.token,
                user: user.try_into()?,
            }),
            None => Err(BlogClientError::InvalidUser),
        }
    }
}

impl TryFrom<blog_grpc::LoginUserResponse> for AuthResponse {
    type Error = BlogClientError;

    fn try_from(response: blog_grpc::LoginUserResponse) -> Result<Self, Self::Error> {
        match response.user {
            Some(user) => Ok(Self {
                token: response.token,
                user: user.try_into()?,
            }),
            None => Err(BlogClientError::InvalidUser),
        }
    }
}

/// Протокол для взаимодействия.
#[derive(Debug)]
pub enum Transport {
    Http(SocketAddr),
    Grpc(SocketAddr),
}

/// Трейт для реализации клиентского взаимодействия.
///
/// Определяет минимальный набор операций, которые должны быть доступны
/// для взаимодействия с сервером независимо от используемого транспорта.
#[async_trait]
pub trait Client {
    type Error;

    /// Регистрация нового пользователя.
    async fn register(
        &mut self,
        username: &str,
        email: &str,
        password: &str,
    ) -> Result<AuthResponse, Self::Error>;

    /// Авторизация пользователя.
    async fn login(&mut self, username: &str, password: &str) -> Result<AuthResponse, Self::Error>;

    /// Создать новый пост.
    async fn create_post(
        &mut self,
        token: &str,
        title: &str,
        content: &str,
    ) -> Result<Post, Self::Error>;

    /// Получить пост по идентификатору.
    async fn get_post(&mut self, id: i64) -> Result<Post, Self::Error>;

    /// Получить список постов с пагинацией.
    async fn get_posts(&mut self, limit: i64, offset: i64) -> Result<Vec<Post>, Self::Error>;

    /// Обновить существующий пост.
    async fn update_post(
        &mut self,
        token: &str,
        id: i64,
        title: Option<String>,
        content: Option<String>,
    ) -> Result<Post, Self::Error>;

    /// Удалить пост.
    async fn delete_post(&mut self, token: &str, id: i64) -> Result<(), Self::Error>;
}

/// Клиент для взаимодействия с серверной частью системы блога.
///
/// Поддерживает несколько транспортных протоколов (HTTP, gRPC)
/// и автоматически управляет JWT-токеном авторизации.
pub struct BlogClient {
    /// Внутренний клиент, реализующий конкретный протокол.
    inner: Box<dyn Client<Error = BlogClientError>>,
    /// Сохраненный JWT-токен для использования в защищенных запросах.
    token: Option<String>,
}

impl BlogClient {
    /// Создать новый клиент с указанным транспортным протоколом.
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

    /// Установить JWT-токен авторизации для последующих защищенных запросов.
    pub fn set_token(&mut self, token: String) {
        self.token = Some(token);
    }

    /// Получить текущий JWT-токен авторизации.
    pub fn get_token(&self) -> Option<String> {
        self.token.clone()
    }

    /// Зарегистрировать нового пользователя и сохранить токен авторизации.
    pub async fn register(
        &mut self,
        username: &str,
        email: &str,
        password: &str,
    ) -> Result<User, BlogClientError> {
        let response = self.inner.register(username, email, password).await?;
        self.set_token(response.token);

        Ok(response.user)
    }

    /// Авторизовать пользователя и сохранить токен авторизации.
    pub async fn login(&mut self, username: &str, password: &str) -> Result<User, BlogClientError> {
        let response = self.inner.login(username, password).await?;

        self.set_token(response.token);

        Ok(response.user)
    }

    /// Создать новый пост от имени авторизованного пользователя.
    pub async fn create_post(
        &mut self,
        title: &str,
        content: &str,
    ) -> Result<Post, BlogClientError> {
        let token = self.get_token().ok_or(BlogClientError::TokenNotFound)?;

        let post = self.inner.create_post(&token, title, content).await?;

        Ok(post)
    }

    /// Получить пост по идентификатору.
    pub async fn get_post(&mut self, id: i64) -> Result<Post, BlogClientError> {
        let post = self.inner.get_post(id).await?;

        Ok(post)
    }

    /// Получить список постов с пагинацией.
    pub async fn get_posts(
        &mut self,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Post>, BlogClientError> {
        let posts = self.inner.get_posts(limit, offset).await?;

        Ok(posts)
    }

    /// Обновить пост от имени авторизованного пользователя.
    pub async fn update_post(
        &mut self,
        id: i64,
        title: Option<String>,
        content: Option<String>,
    ) -> Result<Post, BlogClientError> {
        let token = self.get_token().ok_or(BlogClientError::TokenNotFound)?;

        let post = self.inner.update_post(&token, id, title, content).await?;

        Ok(post)
    }

    /// Удалить пост от имени авторизованного пользователя.
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

    /// Время создания пользователя.
    pub created_at: DateTime<Utc>,
}

impl std::fmt::Display for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            r#"Идентификатор пользователя: {}
Имя пользователя: {}
Email пользователя: {}
Время создания пользователя: {}
"#,
            self.id, self.username, self.email, self.created_at
        )
    }
}

impl TryFrom<blog_grpc::User> for User {
    type Error = BlogClientError;

    fn try_from(user: blog_grpc::User) -> Result<Self, Self::Error> {
        Ok(Self {
            id: user.id,
            username: user.username,
            email: user.email,
            created_at: user
                .created_at
                .parse()
                .map_err(|_| BlogClientError::InvalidUser)?,
        })
    }
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

impl std::fmt::Display for Post {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            r#"Идентификатор поста: {}
Заголовок поста: {}
Содержимое поста: {}
Идентификатор пользователя-автора поста: {}
Время создания поста: {}
Время последнего обновления поста: {}
"#,
            self.id, self.title, self.content, self.author_id, self.created_at, self.updated_at
        )
    }
}

impl TryFrom<blog_grpc::Post> for Post {
    type Error = BlogClientError;

    fn try_from(post: blog_grpc::Post) -> Result<Self, Self::Error> {
        Ok(Self {
            id: post.id,
            title: post.title,
            content: post.content,
            author_id: post.author_id,
            created_at: post
                .created_at
                .parse()
                .map_err(|_| BlogClientError::InvalidPostContent)?,
            updated_at: post
                .updated_at
                .parse()
                .map_err(|_| BlogClientError::InvalidPostContent)?,
        })
    }
}
