use crate::error;

use sqlx::PgPool;
use std::env;
use std::fs;
use std::time::Duration;

const DB_POOL_MAX_OPEN: u32 = 32;
const DB_POOL_MAX_IDLE: u64 = 8;
const DB_POOL_TIMEOUT_SECONDS: u64 = 15;
const INIT_SQL: &str = "./db.sql";

pub async fn create_pool() -> Result<PgPool, sqlx::Error> {
    let db_url = match env::var("DATABASE_URL") {
        Ok(val) => val,
        Err(e) => panic!("Couldn't find DATABASE_URL: {}", e),
    };

    Ok(PgPool::builder()
        .max_size(DB_POOL_MAX_OPEN)
        .idle_timeout(Some(Duration::from_secs(DB_POOL_MAX_IDLE)))
        .connect_timeout(Duration::from_secs(DB_POOL_TIMEOUT_SECONDS))
        .build(&db_url)
        .await?)
}

pub async fn init_db(db_pool: &PgPool) -> Result<(), error::Error> {
    let init_file: String = fs::read_to_string(INIT_SQL)?;
    sqlx::query(&init_file)
        .execute(db_pool)
        .await
        .map_err(error::Error::DBInitError)?;

    Ok(())
}
