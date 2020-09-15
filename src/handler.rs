use crate::data::*;
use crate::db;
use crate::error;
use sqlx::PgPool;
use warp::{http::StatusCode, reject, Rejection, Reply};

pub async fn health_handler(db_pool: PgPool) -> std::result::Result<impl Reply, Rejection> {
    sqlx::query("SELECT 1")
        .execute(&db_pool)
        .await
        .map_err(|e| reject::custom(error::Error::DBQueryError(e)))?;

    Ok(StatusCode::OK)
}

pub async fn create_todo_handler(body: TodoRequest, db_pool: PgPool) -> error::Result<impl Reply> {
    Ok(warp::reply::json(&TodoResponse::of(
        db::create_todo(&db_pool, body).await?,
    )))
}
