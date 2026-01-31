use sqlx::PgPool;
use crate::domain::error::UserError;
use crate::domain::user::User;

#[derive(Debug)]
pub struct UserRepository {
    pool: PgPool
}

impl UserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_user(&self, user: User) -> Result<User, UserError> {
        let post = sqlx::query_as!(
            User,
            "INSERT INTO users (username, email, password_hash) VALUES ($1, $2, $3) RETURNING *",
            user.username, user.email, user.password_hash
        ).fetch_one(&self.pool).await.map_err(
            |e| {
                match e {
                    sqlx::Error::Database(db_err) if db_err.code() == Some(std::borrow::Cow::Borrowed("23505")) => {
                        UserError::UserAlreadyExists
                    }
                    _ => {
                        eprintln!("Database error while creating user: {:?}", e);
                        UserError::Database(e)
                    }
                }
            }
        )?;

        Ok(post)
    }

    pub async fn get_user(&self, username: &str) -> Result<User, UserError> {
        let user = sqlx::query_as!(
            User,
            "SELECT * FROM users WHERE username = $1",
            username
        ).fetch_optional(&self.pool).await?.ok_or(UserError::UserNotFound)?;

        Ok(user)
    }
}
