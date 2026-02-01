use std::net::SocketAddr;
use tonic::{async_trait, Request};
use crate::blog_grpc::blog_service_client::BlogServiceClient;
use crate::blog_grpc::{
    CreatePostRequest, CreateUserRequest, DeletePostRequest,
    GetPostRequest, GetPostsRequest, LoginUserRequest, UpdatePostRequest
};
use crate::error::BlogClientError;
use crate::{Client, Post};

pub struct GrpcClient {
    addr: SocketAddr,
    inner: BlogServiceClient<tonic::transport::Channel>,
}

impl GrpcClient {
    pub async fn new(addr: SocketAddr) -> Result<Self, BlogClientError> {
        let inner = BlogServiceClient::connect(addr.to_string()).await?;

        Ok(Self { addr, inner })
    }
}

#[async_trait]
impl Client for GrpcClient {
    type Error = BlogClientError;

    async fn register(
        &mut self,
        username: &str,
        email: &str,
        password: &str,
    ) -> Result<String, Self::Error> {
        let payload = Request::new(CreateUserRequest {
            username: username.to_string(),
            email: email.to_string(),
            password: password.to_string(),
        });

        let response = self.inner.register(payload).await?.into_inner();


        Ok(response.token)
    }

    async fn login(&mut self, username: &str, password: &str) -> Result<String, Self::Error> {
        let payload = Request::new(LoginUserRequest {
            username: username.to_string(),
            password: password.to_string(),
        });

        let response = self.inner.login(payload).await?.into_inner();

        Ok(response.token)
    }

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

        payload
            .metadata_mut()
            .insert("authorization", token.parse().map_err(|_| BlogClientError::InvalidToken)?);

        let response = self.inner.create_post(payload).await?.into_inner();

        let post = response
            .post
            .ok_or(BlogClientError::PostNotFound)?
            .try_into()?;

        Ok(post)
    }

    async fn get_post(&mut self, id: i64) -> Result<Post, Self::Error> {
        let payload = Request::new(GetPostRequest { id });

        let response = self.inner.get_post(payload).await?.into_inner();

        let post = response
            .post
            .ok_or(BlogClientError::PostNotFound)?
            .try_into()?;

        Ok(post)
    }

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

    async fn update_post(
        &mut self,
        token: &str,
        id: i64,
        title: Option<String>,
        content: Option<String>,
    ) -> Result<Post, Self::Error> {
        let mut payload = Request::new(UpdatePostRequest {
            id,
            title,
            content,
        });

        payload
            .metadata_mut()
            .insert("authorization", token.parse().map_err(|_| BlogClientError::InvalidToken)?);

        let response = self.inner.update_post(payload).await?.into_inner();

        let post = response
            .post
            .ok_or(BlogClientError::PostNotFound)?
            .try_into()?;

        Ok(post)
    }

    async fn delete_post(&mut self, token: &str, id: i64) -> Result<(), Self::Error> {
        let mut payload = Request::new(DeletePostRequest { id });

        payload
            .metadata_mut()
            .insert("authorization", token.parse().map_err(|_| BlogClientError::InvalidToken)?);

        self.inner.delete_post(payload).await?.into_inner();

        Ok(())
    }
}