//! Репозиторий для работы с пользователями в базе данных.

use crate::domain::error::UserError;
use crate::domain::user::User;
use sqlx::PgPool;

/// Репозиторий для работы с пользователями в базе данных.
#[derive(Debug)]
pub(crate) struct UserRepository {
    /// Пул соединений с базой данных PostgreSQL.
    pool: PgPool,
}

impl UserRepository {
    pub(crate) fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Создать нового пользователя.
    pub(crate) async fn create_user(&self, user: User) -> Result<User, UserError> {
        let post = sqlx::query_as!(
            User,
            "INSERT INTO users (username, email, password_hash) VALUES ($1, $2, $3) RETURNING *",
            user.username,
            user.email,
            user.password_hash
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::Database(db_err)
                if db_err.code() == Some(std::borrow::Cow::Borrowed("23505")) =>
            {
                UserError::UserAlreadyExists
            }
            _ => UserError::Database(e),
        })?;

        Ok(post)
    }

    /// Получить пользователя по имени пользователя.
    pub(crate) async fn get_user(&self, username: &str) -> Result<User, UserError> {
        let user = sqlx::query_as!(User, "SELECT * FROM users WHERE username = $1", username)
            .fetch_optional(&self.pool)
            .await?
            .ok_or(UserError::UserNotFound)?;

        Ok(user)
    }
}
