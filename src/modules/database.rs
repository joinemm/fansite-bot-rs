use sqlx::postgres::{PgPool, PgPoolOptions};
use std::error::Error;

pub async fn get_pool(database_url: &str) -> Result<PgPool, Box<dyn Error>> {
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await?;
    Ok(pool)
}
