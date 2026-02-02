use std::collections::HashMap;
use std::net::SocketAddr;
use tonic::{async_trait, Request};
use crate::error::BlogClientError;
use crate::{AuthResponse, Client, Post, Transport};

pub struct HttpClient {
    addr: String,
    inner: reqwest::Client
}

impl HttpClient {
    pub async fn new(addr: SocketAddr) -> Result<Self, BlogClientError> {
        let inner = reqwest::Client::new();

        Ok(Self { addr: format!("http://{addr}"), inner })
    }
}

#[async_trait]
impl Client for HttpClient {
    type Error = BlogClientError;

    async fn register(
        &mut self,
        username: &str,
        email: &str,
        password: &str,
    ) -> Result<String, Self::Error> {
        let endpoint = format!("{}/api/auth/register", self.addr);

        let payload = serde_json::json!({
            "username": username,
            "email": email,
            "password": password
        });

        let response = self.inner
            .post(endpoint)
            .json(&payload)
            .send()
            .await
            .and_then(|resp| resp.error_for_status())?
            .json::<AuthResponse>()
            .await?;

        Ok(response.token)
    }

    async fn login(&mut self, username: &str, password: &str) -> Result<String, Self::Error> {
        let endpoint = format!("{}/api/auth/login", self.addr);

        let payload = serde_json::json!({
            "username": username,
            "password": password
        });

        let response = self.inner
            .post(endpoint)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .and_then(|resp| resp.error_for_status())?
            .json::<AuthResponse>()
            .await?;

        Ok(response.token)
    }

    async fn create_post(&mut self, token: &str, title: &str, content: &str) -> Result<Post, Self::Error> {
        let endpoint = format!("{}/api/auth/register", self.addr);

        let payload = serde_json::json!({
            "title": title,
            "content": content
        });

        let post = self.inner
            .post(endpoint)
            .header("Authorization", format!("Bearer {}", token))
            .json(&payload)
            .send()
            .await
            .and_then(|resp| resp.error_for_status())?
            .json::<Post>()
            .await?;

        Ok(post)
    }

    async fn get_post(&mut self, id: i64) -> Result<Post, Self::Error> {
        let endpoint = format!("{}/api/posts/{id}", self.addr);

        let post = self.inner
            .get(endpoint)
            .send()
            .await
            .and_then(|resp| resp.error_for_status())?
            .json::<Post>()
            .await?;

        Ok(post)
    }

    async fn get_posts(&mut self, limit: i64, offset: i64) -> Result<Vec<Post>, Self::Error> {
        let endpoint = format!("{}/api/posts", self.addr);

        let posts = self.inner
            .get(endpoint)
            .query(&[("limit", limit), ("offset", offset)])
            .send()
            .await
            .and_then(|resp| resp.error_for_status())?
            .json::<Vec<Post>>()
            .await?;

        Ok(posts)
    }

    async fn update_post(&mut self, token: &str, id: i64, title: Option<String>, content: Option<String>) -> Result<Post, Self::Error> {
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

        let post = self.inner
            .put(endpoint)
            .header("Authorization", format!("Bearer {}", token))
            .json(&payload)
            .send()
            .await
            .and_then(|resp| resp.error_for_status())?
            .json::<Post>()
            .await?;

        Ok(post)
    }

    async fn delete_post(&mut self, token: &str, id: i64) -> Result<(), Self::Error> {
        let endpoint = format!("{}/api/posts/{id}", self.addr);

        self.inner
            .delete(endpoint)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await?;

        Ok(())
    }
}
