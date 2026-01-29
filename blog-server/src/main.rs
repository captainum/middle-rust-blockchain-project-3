mod domain;
mod infrastructure;

use infrastructure::database::{create_pool, run_migrations};

mod blog_grpc {
    tonic::include_proto!("blog");
}

#[tokio::main]
async fn main() -> anyhow::Result<()>{
    dotenv::dotenv()?;

    let pool = create_pool().await?;
    run_migrations(&pool).await?;

    Ok(())
}
