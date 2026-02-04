//! Модуль взаимодействия с JWT-токенами.

use serde::{Deserialize, Serialize};
use std::time::Duration;

use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use sqlx::types::chrono::Utc;

/// Загрузить JWT-токен из переменной окружения.
pub(crate) fn load_secret() -> anyhow::Result<String> {
    let secret = std::env::var("JWT_SECRET").map_err(|e| anyhow::anyhow!("JWT_SECRET: {e}"))?;

    if secret.len() < 32 {
        anyhow::bail!("JWT_SECRET must be less than 32 characters");
    }

    Ok(secret)
}

/// Аттрибуты пользователя.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Claims {
    /// Идентификатор пользователя.
    pub user_id: i64,

    /// Имя пользователя.
    pub username: String,

    /// Время истечения токена.
    pub exp: usize,
}

/// Сервис взаимодействия с JWT-токенами.
#[derive(Debug)]
pub(crate) struct JwtService {
    /// Ключ шифрования.
    encoding: EncodingKey,

    /// Ключ расшифрования.
    decoding: DecodingKey,
}

impl JwtService {
    /// Создание сервиса из секретного ключа.
    pub(crate) fn new(secret: &str) -> Self {
        let (encoding, decoding) = (
            EncodingKey::from_secret(secret.as_bytes()),
            DecodingKey::from_secret(secret.as_bytes()),
        );

        Self { encoding, decoding }
    }

    /// Генерация JWT-токена с временем жизни 24 часа.
    pub(crate) fn generate_token(&self, user_id: i64, username: &str) -> anyhow::Result<String> {
        let exp = (Utc::now() + Duration::from_secs(24 * 60 * 60)).timestamp() as usize;
        let claims = Claims {
            user_id,
            username: username.to_string(),
            exp,
        };

        let header = Header::default();

        let token = encode(&header, &claims, &self.encoding)?;

        Ok(token)
    }

    /// Проверка и декодирование токена.
    pub(crate) fn verify_token(&self, token: &str) -> anyhow::Result<Claims> {
        let validator = Validation::default();

        let decoded = decode::<Claims>(token, &self.decoding, &validator)?;

        Ok(decoded.claims)
    }
}
