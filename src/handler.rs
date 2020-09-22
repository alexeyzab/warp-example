use crate::{data::*, db, error, error::Error::*};
use serde_derive::Deserialize;
use sqlx::PgPool;
use warp::{http::StatusCode, reject, reply::json, Rejection, Reply};

#[derive(Deserialize)]
pub struct SearchQuery {
    search: Option<String>,
}

pub async fn health_handler(db_pool: PgPool) -> std::result::Result<impl Reply, Rejection> {
    sqlx::query("SELECT 1")
        .execute(&db_pool)
        .await
        .map_err(|e| reject::custom(DBQueryError(e)))?;

    Ok(StatusCode::OK)
}

pub async fn create_todo_handler(body: TodoRequest, db_pool: PgPool) -> error::Result<impl Reply> {
    Ok(json(&TodoResponse::of(
        db::create_todo(&db_pool, body).await?,
    )))
}

pub async fn list_todos_handler(query: SearchQuery, db_pool: PgPool) -> error::Result<impl Reply> {
    let todos = db::fetch_todos(&db_pool, query.search).await?;
    Ok(json::<Vec<_>>(
        &todos.into_iter().map(TodoResponse::of).collect(),
    ))
}

pub async fn update_todo_handler(
    id: i32,
    body: TodoUpdateRequest,
    db_pool: PgPool,
) -> error::Result<impl Reply> {
    Ok(json(&TodoResponse::of(
        db::update_todo(&db_pool, id, body).await?,
    )))
}

pub async fn delete_todo_handler(id: i32, db_pool: PgPool) -> error::Result<impl Reply> {
    db::delete_todo(&db_pool, id).await?;
    Ok(StatusCode::OK)
}
