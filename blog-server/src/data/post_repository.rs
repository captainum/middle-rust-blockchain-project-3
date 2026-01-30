use sqlx::{PgPool, QueryBuilder, Postgres, Executor, Transaction};
use crate::domain::error::PostError;
use crate::domain::post::{Post, UpdatePostRequest};

#[derive(Debug)]
pub struct PostRepository {
    pool: PgPool
}

impl PostRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn begin_transaction(&self) -> Result<Transaction<'static, Postgres>, sqlx::Error> {
        self.pool.begin().await
    }

    pub async fn commit_transaction(&self, tx: Transaction<'static, Postgres>) -> Result<(), sqlx::Error> {
        tx.commit().await
    }

    pub async fn create_post(&self, post: Post, author_id: i64) -> Result<Post, PostError> {
        self.create_post_with_tx(post, author_id, &self.pool).await
    }

    pub async fn create_post_with_tx<'e, E>(&self, post: Post, author_id: i64, executor: E) -> Result<Post, PostError>
    where
        E: Executor<'e, Database = Postgres> {
        let post = sqlx::query_as!(
            Post,
            "INSERT INTO posts (title, content, author_id) VALUES ($1, $2, $3) RETURNING *",
            post.title,
            post.content,
            author_id
        ).fetch_one(executor).await?;

        Ok(post)
    }

    pub async fn get_post(&self, id: i64) -> Result<Post, PostError> {
        self.get_post_with_tx(id, &self.pool).await
    }

    pub async fn get_post_with_tx<'e, E>(&self, id: i64, executor: E) -> Result<Post, PostError>
    where
        E: Executor<'e, Database = Postgres>,
    {
        let post = sqlx::query_as!(
            Post,
            "SELECT * FROM posts WHERE id = $1",
            id
        ).fetch_optional(executor).await?.ok_or(PostError::PostNotFound)?;

        Ok(post)
    }

    pub async fn get_posts(&self, limit: i64, offset: i64) -> Result<Vec<Post>, PostError> {
        self.get_posts_with_tx(limit, offset, &self.pool).await
    }

    pub async fn get_posts_with_tx<'e, E>(&self, limit: i64, offset: i64, executor: E) -> Result<Vec<Post>, PostError>
    where
        E: Executor<'e, Database = Postgres>,
    {
        let posts = sqlx::query_as!(
            Post,
            "SELECT * FROM posts LIMIT $1 OFFSET $2",
            limit, offset
        ).fetch_all(executor).await?;

        Ok(posts)
    }

    pub async fn update_post(&self, post: UpdatePostRequest) -> Result<Post, PostError> {
        self.update_post_with_tx(post, &self.pool).await
    }

    pub async fn update_post_with_tx<'e, E>(&self, post: UpdatePostRequest, executor: E) -> Result<Post, PostError>
    where
        E: Executor<'e, Database = Postgres>,
    {
        let mut query_builder = QueryBuilder::new("UPDATE posts SET ");

        let mut has_fields = false;

        if let Some(title) = &post.title {
            if has_fields {
                query_builder.push(", ");
            }
            query_builder.push("title = ");
            query_builder.push_bind(title);
            has_fields = true;
        }

        if let Some(content) = &post.content {
            if has_fields {
                query_builder.push(", ");
            }
            query_builder.push("content = ");
            query_builder.push_bind(content);
            has_fields = true;
        }

        if has_fields {
            query_builder.push(", ");
        }

        query_builder.push("updated_at = NOW() WHERE id = ");
        query_builder.push_bind(post.id);
        query_builder.push(" RETURNING *");

        let updated_post = query_builder
            .build_query_as::<Post>()
            .fetch_optional(executor)
            .await?
            .ok_or(PostError::PostNotFound)?;

        Ok(updated_post)
    }

    pub async fn delete_post(&self, id: i64) -> Result<(), PostError> {
        self.delete_post_with_tx(id, &self.pool).await
    }

    pub async fn delete_post_with_tx<'e, E>(&self, id: i64, executor: E) -> Result<(), PostError>
    where
        E: Executor<'e, Database = Postgres>,
    {
        sqlx::query!("DELETE FROM posts WHERE id = $1", id)
            .execute(executor)
            .await?;

        Ok(())
    }
}