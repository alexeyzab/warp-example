use sqlx::PgPool;
use std::convert::Infallible;
use warp::Filter;

mod data;
mod db;
mod error;
mod handler;

#[tokio::main]
async fn main() {
    let db_pool = db::create_pool()
        .await
        .expect("database pool can be created");

    db::init_db(&db_pool)
        .await
        .expect("database can be initialized");

    let health_route = warp::path!("health")
        .and(with_db(db_pool.clone()))
        .and_then(handler::health_handler);

    let todo = warp::path("todo");

    let todo_routes = todo
        .and(warp::get())
        .and(warp::query())
        .and(with_db(db_pool.clone()))
        .and_then(handler::list_todos_handler)
        .or(todo
            .and(warp::post())
            .and(warp::body::json())
            .and(with_db(db_pool.clone()))
            .and_then(handler::create_todo_handler))
        .or(todo
            .and(warp::put())
            .and(warp::path::param())
            .and(warp::body::json())
            .and(with_db(db_pool.clone()))
            .and_then(handler::update_todo_handler))
        .or(todo
            .and(warp::delete())
            .and(warp::path::param())
            .and(with_db(db_pool.clone()))
            .and_then(handler::delete_todo_handler));

    let routes = health_route
        .or(todo_routes)
        .with(warp::cors().allow_any_origin())
        .recover(error::handle_rejection);

    warp::serve(routes).run(([127, 0, 0, 1], 8000)).await;
}

fn with_db(db_pool: PgPool) -> impl Filter<Extract = (PgPool,), Error = Infallible> + Clone {
    warp::any().map(move || db_pool.clone())
}
