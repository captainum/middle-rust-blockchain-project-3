//! Модуль взаимодействия с JWT-токенами.

use std::time::Duration;
use serde::{Deserialize, Serialize};

use jsonwebtoken::{EncodingKey, DecodingKey, Header, Validation, encode, decode};
use sqlx::types::chrono::Utc;

/// Аттрибуты пользователя.
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// Идентификатор пользователя.
    pub user_id: i64,

    /// Имя пользователя.
    pub username: String,

    /// Время истечения токена.
    pub exp: usize,
}

/// Сервис взаимодействия с JWT-токенами.
#[derive(Debug)]
pub struct JwtService {
    /// Ключ шифрования.
    encoding: EncodingKey,

    /// Ключ расшифрования.
    decoding: DecodingKey,
}

impl JwtService {
    /// Создание сервиса из секретного ключа.
    pub fn new(secret: &str) -> Self {
        let (encoding, decoding) = (
            EncodingKey::from_secret(secret.as_bytes()),
            DecodingKey::from_secret(secret.as_bytes())
        );

        Self { encoding, decoding }
    }

    /// Генерация JWT-токена с временем жизни 24 часа.
    pub fn generate_token(&self, user_id: i64, username: &str) -> anyhow::Result<String> {
        let exp = (Utc::now() + Duration::from_secs(24 * 60 * 60)).timestamp() as usize;
        let claims = Claims {
            user_id,
            username: username.to_string(),
            exp
        };

        let header = Header::default();

        let token = encode(&header, &claims, &self.encoding)?;

        Ok(token)
    }

    /// Проверка и декодирование токена.
    pub fn verify_token(&self, token: &str) -> anyhow::Result<Claims> {
        let validator = Validation::default();

        let decoded = decode::<Claims>(token, &self.decoding, &validator)?;

        Ok(decoded.claims)
    }
}