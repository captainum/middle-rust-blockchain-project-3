use crate::data::post_repository::PostRepository;
use crate::domain::error::PostError;
use crate::domain::post::{CreatePostRequest, Post, UpdatePostRequest};
use sqlx::{Postgres, Transaction};
use std::sync::Arc;

/// Сервис для управления постами блога.
#[derive(Debug)]
pub(crate) struct BlogService {
    /// Репозиторий для работы с постами.
    post_repository: Arc<PostRepository>,
}

impl BlogService {
    /// Создать новый экземпляр сервиса блога.
    pub(crate) fn new(post_repository: Arc<PostRepository>) -> Self {
        Self { post_repository }
    }

    /// Создать новый пост.
    pub(crate) async fn create_post(
        &self,
        post: CreatePostRequest,
        author_id: i64,
    ) -> Result<Post, PostError> {
        self.post_repository
            .create_post(post.into(), author_id)
            .await
    }

    /// Получить пост по идентификатору.
    pub(crate) async fn get_post(&self, id: i64) -> Result<Post, PostError> {
        self.post_repository.get_post(id).await
    }

    /// Получить список постов с пагинацией.
    pub(crate) async fn get_posts(&self, limit: i64, offset: i64) -> Result<Vec<Post>, PostError> {
        self.post_repository.get_posts(limit, offset).await
    }

    /// Обновить существующий пост.
    pub(crate) async fn update_post(
        &self,
        post: UpdatePostRequest,
        user_id: i64,
    ) -> Result<Post, PostError> {
        let mut tx = self.post_repository.begin_transaction().await?;

        if !self.is_author(post.id, user_id, &mut tx).await? {
            return Err(PostError::Forbidden);
        }

        let post = self.post_repository.update_post(post).await?;

        tx.commit().await?;

        Ok(post)
    }

    /// Удалить пост.
    pub(crate) async fn delete_post(&self, id: i64, user_id: i64) -> Result<(), PostError> {
        let mut tx = self.post_repository.begin_transaction().await?;

        if !self.is_author(id, user_id, &mut tx).await? {
            return Err(PostError::Forbidden);
        }

        self.post_repository
            .delete_post_with_tx(id, &mut *tx)
            .await?;

        tx.commit().await?;

        Ok(())
    }

    /// Проверить авторство пользователя для данного поста.
    async fn is_author(
        &self,
        post_id: i64,
        user_id: i64,
        tx: &mut Transaction<'static, Postgres>,
    ) -> Result<bool, PostError> {
        let author_id = self
            .post_repository
            .get_post_with_tx(post_id, &mut **tx)
            .await?
            .author_id;

        Ok(user_id == author_id)
    }
}
