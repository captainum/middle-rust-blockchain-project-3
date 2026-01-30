use std::sync::Arc;
use crate::data::post_repository::PostRepository;
use crate::domain::error::PostError;
use crate::domain::post::{CreatePostRequest, Post, UpdatePostRequest};

pub struct BlogService {
    post_repository: Arc<PostRepository>,
}

impl BlogService {
    pub async fn create_post(&self, post: CreatePostRequest, author_id: i64) -> Result<Post, PostError> {
        self.post_repository.create_post(post.into(), author_id).await
    }

    pub async fn get_post(&self, id: i64) -> Result<Post, PostError> {
        self.post_repository.get_post(id).await
    }

    pub async fn get_posts(&self, limit: i64, offset: i64) -> Result<Vec<Post>, PostError> {
        self.post_repository.get_posts(limit, offset).await
    }

    pub async fn update_post(&self, post: UpdatePostRequest) -> Result<Post, PostError> {
        self.post_repository.update_post(post).await
    }

    pub async fn delete_post(&self, id: i64) -> Result<(), PostError> {
        self.post_repository.delete_post(id).await
    }
}