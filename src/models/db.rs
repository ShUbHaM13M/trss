use async_sqlite::{Error, Pool, PoolBuilder};

const DATABASE_URL: &str = "trss.db";

#[derive(Clone)]
pub struct Database {
    pub pool: Pool,
}

impl Database {
    pub async fn init() -> Result<Self, Error> {
        let pool = PoolBuilder::new().path(DATABASE_URL).open().await?;
        Ok(Self { pool })
    }
}
