//! Модуль начального взаимодействия с БД.

use std::env;
use std::time::Duration;
use sqlx::{postgres::PgPoolOptions, PgPool, migrate};

/// Создать пул соединений.
pub async fn create_pool() -> anyhow::Result<PgPool> {
    let database_url = env::var("DATABASE_URL")?;

    let pool = PgPoolOptions::new()
        .max_connections(20)
        .min_connections(5)
        .acquire_timeout(Duration::from_secs(5))
        .connect(&database_url)
        .await?;

    Ok(pool)
}

/// Актуализировать миграции в БД.
pub async fn run_migrations(pool: &PgPool) -> anyhow::Result<()> {
    migrate!().run(pool).await?;

    Ok(())
}
