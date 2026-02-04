//! gRPC-клиент для API сервиса блога.

use crate::blog_grpc::blog_service_client::BlogServiceClient;
use crate::blog_grpc::{
    CreatePostRequest, CreateUserRequest, DeletePostRequest, GetPostRequest, GetPostsRequest,
    LoginUserRequest, UpdatePostRequest,
};
use crate::error::BlogClientError;
use crate::{AuthResponse, Client, Post};
use std::net::SocketAddr;
use tonic::{Request, async_trait};

/// gRPC-клиент для взаимодействия с сервисом блога.
#[derive(Clone)]
pub(crate) struct GrpcClient {
    /// Адрес сервера.
    #[allow(dead_code)]
    addr: String,
    /// Внутренний gRPC-клиент для отправки запросов.
    inner: BlogServiceClient<tonic::transport::Channel>,
}

impl GrpcClient {
    /// Создать новый экземпляр gRPC-клиента и подключиться к серверу.
    pub(crate) async fn new(addr: SocketAddr) -> Result<Self, BlogClientError> {
        let addr = format!("http://{addr}");
        let inner = BlogServiceClient::connect(addr.clone()).await?;

        Ok(Self { addr, inner })
    }
}

/// Реализация клиентского интерфейса для gRPC.
#[async_trait]
impl Client for GrpcClient {
    type Error = BlogClientError;

    /// Регистрация нового пользователя.
    async fn register(
        &mut self,
        username: &str,
        email: &str,
        password: &str,
    ) -> Result<AuthResponse, Self::Error> {
        let payload = Request::new(CreateUserRequest {
            username: username.to_string(),
            email: email.to_string(),
            password: password.to_string(),
        });

        let response = self.inner.register(payload).await?.into_inner();

        Ok(response.try_into()?)
    }

    /// Авторизация пользователя.
    async fn login(&mut self, username: &str, password: &str) -> Result<AuthResponse, Self::Error> {
        let payload = Request::new(LoginUserRequest {
            username: username.to_string(),
            password: password.to_string(),
        });

        let response = self
            .inner
            .login(payload)
            .await
            .map_err(|status| {
                let code = status.code();
                match code {
                    tonic::Code::NotFound => BlogClientError::UserNotFound,
                    tonic::Code::InvalidArgument => BlogClientError::InvalidCredentials,
                    _ => BlogClientError::GrpcStatus(status),
                }
            })?
            .into_inner();

        Ok(response.try_into()?)
    }

    /// Создать новый пост.
    async fn create_post(
        &mut self,
        token: &str,
        title: &str,
        content: &str,
    ) -> Result<Post, Self::Error> {
        let mut payload = Request::new(CreatePostRequest {
            title: title.to_string(),
            content: content.to_string(),
        });

        payload.metadata_mut().insert(
            "authorization",
            format!("Bearer {token}")
                .parse()
                .map_err(|_| BlogClientError::InvalidToken)?,
        );

        let response = self
            .inner
            .create_post(payload)
            .await
            .map_err(check_post_auth_err)?
            .into_inner();

        let post = response
            .post
            .ok_or(BlogClientError::PostNotFound)?
            .try_into()?;

        Ok(post)
    }

    /// Получить пост по идентификатору.
    async fn get_post(&mut self, id: i64) -> Result<Post, Self::Error> {
        let payload = Request::new(GetPostRequest { id });

        let response = self
            .inner
            .get_post(payload)
            .await
            .map_err(|status| {
                let code = status.code();
                match code {
                    tonic::Code::NotFound => BlogClientError::PostNotFound,
                    _ => BlogClientError::GrpcStatus(status),
                }
            })?
            .into_inner();

        let post = response
            .post
            .ok_or(BlogClientError::PostNotFound)?
            .try_into()?;

        Ok(post)
    }

    /// Получить список постов с пагинацией.
    async fn get_posts(&mut self, limit: i64, offset: i64) -> Result<Vec<Post>, Self::Error> {
        let payload = Request::new(GetPostsRequest { limit, offset });

        let response = self.inner.get_posts(payload).await?.into_inner();

        let mut posts = vec![];

        for post in response.posts {
            let p = post.try_into()?;
            posts.push(p);
        }

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
        let mut payload = Request::new(UpdatePostRequest { id, title, content });

        payload.metadata_mut().insert(
            "authorization",
            format!("Bearer {token}")
                .parse()
                .map_err(|_| BlogClientError::InvalidToken)?,
        );

        let response = self
            .inner
            .update_post(payload)
            .await
            .map_err(check_post_auth_err)?
            .into_inner();

        let post = response
            .post
            .ok_or(BlogClientError::PostNotFound)?
            .try_into()?;

        Ok(post)
    }

    /// Удалить пост.
    async fn delete_post(&mut self, token: &str, id: i64) -> Result<(), Self::Error> {
        let mut payload = Request::new(DeletePostRequest { id });

        payload.metadata_mut().insert(
            "authorization",
            format!("Bearer {token}")
                .parse()
                .map_err(|_| BlogClientError::InvalidToken)?,
        );

        self.inner
            .delete_post(payload)
            .await
            .map_err(check_post_auth_err)?
            .into_inner();

        Ok(())
    }
}

/// Преобразовать ошибку gRPC при работе с постами в ошибку клиента.
fn check_post_auth_err(status: tonic::Status) -> BlogClientError {
    let code = status.code();
    match code {
        tonic::Code::Unauthenticated => BlogClientError::UserUnauthorized,
        tonic::Code::NotFound => BlogClientError::PostNotFound,
        tonic::Code::InvalidArgument => BlogClientError::Forbidden,
        _ => BlogClientError::GrpcStatus(status),
    }
}
