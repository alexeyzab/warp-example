use crate::data::*;
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

pub async fn create_todo(db_pool: &PgPool, body: TodoRequest) -> error::Result<Todo> {
    let row = sqlx::query_as!(
        Todo,
        "
      INSERT INTO todo ( name )
      VALUES ( $1 ) RETURNING *
      ",
        &body.name
    )
    .fetch_one(db_pool)
    .await;

    Ok(row.map_err(|e| warp::reject::custom(error::Error::DBQueryError(e)))?)
}

pub async fn fetch_todos(db_pool: &PgPool, search: Option<String>) -> error::Result<Vec<Todo>> {
    let q = match search {
        Some(v) => sqlx::query_as!(Todo, "SELECT id, name, created_at, checked FROM todo WHERE name like $1 ORDER BY created_at DESC", v).fetch_all(db_pool).await,
        None => sqlx::query_as!(Todo, "SELECT id, name, created_at, checked FROM todo ORDER BY created_at DESC").fetch_all(db_pool).await,
    };

    Ok(q.map_err(|e| warp::reject::custom(error::Error::DBQueryError(e)))?)
}

pub async fn update_todo(
    db_pool: &PgPool,
    id: i32,
    body: TodoUpdateRequest,
) -> error::Result<Todo> {
    let row = sqlx::query_as!(
        Todo,
        "UPDATE todo SET name = $1, checked = $2 WHERE id = $3 RETURNING *",
        body.name,
        body.checked,
        id
    )
    .fetch_one(db_pool)
    .await;

    Ok(row.map_err(|e| warp::reject::custom(error::Error::DBQueryError(e)))?)
}

pub async fn delete_todo(db_pool: &PgPool, id: i32) -> error::Result<u64> {
    Ok(sqlx::query!("DELETE FROM todo WHERE id = $1", id)
        .execute(db_pool)
        .await
        .map_err(|e| warp::reject::custom(error::Error::DBQueryError(e)))?)
}
