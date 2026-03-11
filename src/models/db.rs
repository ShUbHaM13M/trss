use std::error::Error;

use async_sqlite::{Pool, PoolBuilder};

use crate::models::{init, settings::Settings, theme::Theme};

const DATABASE_URL: &str = "trss.db";

#[derive(Clone)]
pub struct Database {
    pub pool: Pool,
}

impl Database {
    pub async fn init() -> Result<Self, Box<dyn Error>> {
        let pool = PoolBuilder::new().path(DATABASE_URL).open().await?;
        let db = Self { pool };
        for query in init::SCHEMA_QUERIES {
            db.pool
                .conn(move |conn| {
                    conn.execute(query, [])?;
                    Ok(())
                })
                .await?;
        }
        Theme::init(&db).await?;
        Settings::init(&db).await?;
        Ok(db)
    }
}
