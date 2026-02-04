//! HTTP-клиент для API сервиса блога.

use crate::error::BlogClientError;
use crate::{AuthResponse, Client, Post};
use std::collections::HashMap;
use std::net::SocketAddr;
use tonic::async_trait;

/// HTTP-клиент для взаимодействия с сервисом блога.
#[derive(Clone)]
pub(crate) struct HttpClient {
    /// Адрес сервера.
    addr: String,
    /// Внутренний HTTP-клиент для отправки запросов.
    inner: reqwest::Client,
}

impl HttpClient {
    /// Создать новый экземпляр HTTP-клиента.
    pub(crate) async fn new(addr: SocketAddr) -> Result<Self, BlogClientError> {
        let inner = reqwest::Client::new();

        Ok(Self {
            addr: format!("http://{addr}"),
            inner,
        })
    }
}

/// Реализация клиентского интерфейса для HTTP.
#[async_trait]
impl Client for HttpClient {
    type Error = BlogClientError;

    /// Регистрация нового пользователя.
    async fn register(
        &mut self,
        username: &str,
        email: &str,
        password: &str,
    ) -> Result<AuthResponse, Self::Error> {
        let endpoint = format!("{}/api/auth/register", self.addr);

        let payload = serde_json::json!({
            "username": username,
            "email": email,
            "password": password
        });

        let response = self
            .inner
            .post(endpoint)
            .json(&payload)
            .send()
            .await
            .and_then(|resp| resp.error_for_status())
            .map_err(|err| match err.status() {
                Some(status) => match status {
                    reqwest::StatusCode::BAD_REQUEST => {
                        BlogClientError::InvalidRegistrationCredentials
                    }
                    reqwest::StatusCode::CONFLICT => BlogClientError::UserAlreadyExists,
                    _ => BlogClientError::Http(err),
                },
                None => BlogClientError::Http(err),
            })?
            .json::<AuthResponse>()
            .await?;

        Ok(response)
    }

    /// Авторизация пользователя.
    async fn login(&mut self, username: &str, password: &str) -> Result<AuthResponse, Self::Error> {
        let endpoint = format!("{}/api/auth/login", self.addr);

        let payload = serde_json::json!({
            "username": username,
            "password": password
        });

        let response = self
            .inner
            .post(endpoint)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .and_then(|resp| resp.error_for_status())
            .map_err(|err| match err.status() {
                Some(status) => match status {
                    reqwest::StatusCode::NOT_FOUND => BlogClientError::UserNotFound,
                    reqwest::StatusCode::UNAUTHORIZED => BlogClientError::InvalidCredentials,
                    _ => BlogClientError::Http(err),
                },
                None => BlogClientError::Http(err),
            })?
            .json::<AuthResponse>()
            .await?;

        Ok(response)
    }

    /// Создать новый пост.
    async fn create_post(
        &mut self,
        token: &str,
        title: &str,
        content: &str,
    ) -> Result<Post, Self::Error> {
        let endpoint = format!("{}/api/posts", self.addr);

        let payload = serde_json::json!({
            "title": title,
            "content": content
        });

        let post = self
            .inner
            .post(endpoint)
            .header("Authorization", format!("Bearer {}", token))
            .json(&payload)
            .send()
            .await
            .and_then(|resp| resp.error_for_status())
            .map_err(check_post_auth_err)?
            .json::<Post>()
            .await?;

        Ok(post)
    }

    /// Получить пост по идентификатору.
    async fn get_post(&mut self, id: i64) -> Result<Post, Self::Error> {
        let endpoint = format!("{}/api/posts/{id}", self.addr);

        let post = self
            .inner
            .get(endpoint)
            .send()
            .await
            .and_then(|resp| resp.error_for_status())
            .map_err(|err| match err.status() {
                Some(reqwest::StatusCode::NOT_FOUND) => BlogClientError::PostNotFound,
                _ => BlogClientError::Http(err),
            })?
            .json::<Post>()
            .await?;

        Ok(post)
    }

    /// Получить список постов с пагинацией.
    async fn get_posts(&mut self, limit: i64, offset: i64) -> Result<Vec<Post>, Self::Error> {
        let endpoint = format!("{}/api/posts", self.addr);

        let posts = self
            .inner
            .get(endpoint)
            .query(&[("limit", limit), ("offset", offset)])
            .send()
            .await
            .and_then(|resp| resp.error_for_status())?
            .json::<Vec<Post>>()
            .await?;

        Ok(posts)
    }

    /// Обновить существующий пост.
    async fn update_post(
        &mut self,
        token: &str,
        id: i64,
        title: Option<String>,
        content: Option<String>,
    ) -> Result<Post, Self::Error> {
        let endpoint = format!("{}/api/posts/{id}", self.addr);

        let payload = {
            let mut payload = HashMap::new();

            if let Some(title) = title {
                payload.insert("title", title);
            }

            if let Some(content) = content {
                payload.insert("content", content);
            }

            serde_json::json!(payload)
        };

        let post = self
            .inner
            .put(endpoint)
            .header("Authorization", format!("Bearer {}", token))
            .json(&payload)
            .send()
            .await
            .and_then(|resp| resp.error_for_status())
            .map_err(check_post_auth_err)?
            .json::<Post>()
            .await?;

        Ok(post)
    }

    /// Удалить пост.
    async fn delete_post(&mut self, token: &str, id: i64) -> Result<(), Self::Error> {
        let endpoint = format!("{}/api/posts/{id}", self.addr);

        self.inner
            .delete(endpoint)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .and_then(|resp| resp.error_for_status())
            .map_err(check_post_auth_err)?;

        Ok(())
    }
}

/// Преобразовать ошибку HTTP при работе с постами в ошибку клиента.
fn check_post_auth_err(err: reqwest::Error) -> BlogClientError {
    match err.status() {
        Some(status) => match status {
            reqwest::StatusCode::UNAUTHORIZED => BlogClientError::UserUnauthorized,
            reqwest::StatusCode::NOT_FOUND => BlogClientError::PostNotFound,
            reqwest::StatusCode::FORBIDDEN => BlogClientError::Forbidden,
            _ => BlogClientError::Http(err),
        },
        None => BlogClientError::Http(err),
    }
}
