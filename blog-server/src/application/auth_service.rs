use std::sync::Arc;
use crate::domain::error::UserError;
use crate::domain::user::{CreateUserRequest, CreateUserResponse, LoginUserRequest, LoginUserResponse};

use argon2::{password_hash::PasswordHash, Argon2, PasswordVerifier};
use crate::data::user_repository::UserRepository;
use crate::infrastructure::jwt::JwtService;

#[derive(Debug)]
pub struct AuthService {
    jwt_service: Arc<JwtService>,
    user_repository: Arc<UserRepository>
}

impl AuthService {
    pub fn new(jwt_service: Arc<JwtService>, user_repository: Arc<UserRepository>) -> Self {
        Self {
            jwt_service,
            user_repository
        }
    }

    pub async fn register(&self, user: CreateUserRequest) -> Result<CreateUserResponse, UserError> {
        let user = self.user_repository.create_user(
            user.try_into()?
        ).await?;

        let token = self.jwt_service.generate_token(user.id, &user.username)
            .map_err(|e| UserError::CreateJwtToken(e.to_string()))?;

        Ok(CreateUserResponse { token })
    }

    pub async fn login(&self, request: LoginUserRequest) -> Result<LoginUserResponse, UserError> {
        let user = self.user_repository.get_user(&request.username).await?;

        let parsed_hash = PasswordHash::new(&user.password_hash)?;

        Argon2::default()
            .verify_password(request.password.as_bytes(), &parsed_hash)
            .map_err(|_| UserError::InvalidCredentials)?;

        let token = self.jwt_service.generate_token(user.id, &user.username)
            .map_err(|e| UserError::CreateJwtToken(e.to_string()))?;

        Ok(LoginUserResponse { token })
    }
}
