use crate::config::Config;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};

#[derive(Clone)]
pub struct AppData {
    pub pool: Pool<Postgres>,
    pub config: Config,
}

impl AppData {
    pub async fn new(config: Config) -> anyhow::Result<AppData> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&config.database_url)
            .await?;

        Ok(AppData { pool, config })
    }
}
