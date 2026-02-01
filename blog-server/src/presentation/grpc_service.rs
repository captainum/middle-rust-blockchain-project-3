use tonic::{Request, Response, Status};
use crate::blog_grpc::blog_service_server::BlogService;
use crate::blog_grpc::{CreatePostRequest, CreatePostResponse, CreateUserRequest, CreateUserResponse, DeletePostRequest, DeletePostResponse, GetPostRequest, GetPostResponse, GetPostsRequest, GetPostsResponse, LoginUserRequest, LoginUserResponse, UpdatePostRequest, UpdatePostResponse};
use crate::presentation::AppState;

fn extract_user_id(request: &tonic::metadata::MetadataMap, jwt_service: &crate::infrastructure::jwt::JwtService) -> Result<i64, Status> {
    let token = request
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .ok_or(Status::unauthenticated("Missing or invalid authorization header"))?;

    let claims = jwt_service
        .verify_token(token)
        .map_err(|_| Status::unauthenticated("Invalid token"))?;

    Ok(claims.user_id)
}

#[derive(Debug, Clone)]
pub struct BlogGrpcService {
    state: AppState,
}

impl BlogGrpcService {
    pub fn new(state: AppState) -> Self {
        Self { state }
    }
}

#[tonic::async_trait]
impl BlogService for BlogGrpcService {
    async fn register(&self, request: Request<CreateUserRequest>) -> Result<Response<CreateUserResponse>, Status> {
        let request = request.into_inner().into();

        Ok(Response::new(self.state.auth_service.register(request).await?.into()))
    }

    async fn login(&self, request: Request<LoginUserRequest>) -> Result<Response<LoginUserResponse>, Status> {
        let request = request.into_inner().into();

        Ok(Response::new(self.state.auth_service.login(request).await?.into()))
    }

    async fn create_post(&self, request: Request<CreatePostRequest>) -> Result<Response<CreatePostResponse>, Status> {
        let user_id = extract_user_id(request.metadata(), &self.state.jwt_service)?;
        let request = request.into_inner().into();

        let post = self.state.blog_service.create_post(request, user_id).await?;

        Ok(Response::new(CreatePostResponse { post: Some(post.into()) }))
    }

    async fn get_post(&self, request: Request<GetPostRequest>) -> Result<Response<GetPostResponse>, Status> {
        let request = request.into_inner();
        let post = self.state.blog_service.get_post(request.id).await?;

        Ok(Response::new(GetPostResponse { post: Some(post.into()) }))
    }

    async fn get_posts(&self, request: Request<GetPostsRequest>) -> Result<Response<GetPostsResponse>, Status> {
        let request = request.into_inner();
        let posts = self.state.blog_service.get_posts(request.limit, request.offset).await?;

        Ok(Response::new(GetPostsResponse { posts: posts.into_iter().map(|p| p.into()).collect() }))
    }

    async fn update_post(&self, request: Request<UpdatePostRequest>) -> Result<Response<UpdatePostResponse>, Status> {
        let user_id = extract_user_id(request.metadata(), &self.state.jwt_service)?;
        let request = request.into_inner().into();

        let post = self.state.blog_service.update_post(request, user_id).await?;

        Ok(Response::new(UpdatePostResponse { post: Some(post.into()) }))
    }

    async fn delete_post(&self, request: Request<DeletePostRequest>) -> Result<Response<DeletePostResponse>, Status> {
        let user_id = extract_user_id(request.metadata(), &self.state.jwt_service)?;
        let request = request.into_inner();

        self.state.blog_service.delete_post(request.id, user_id).await?;

        Ok(Response::new(DeletePostResponse {}))
    }
}