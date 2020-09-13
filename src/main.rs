mod data;
mod db;
mod error;
mod handler;

use std::convert::Infallible;

use sqlx::PgPool;
use warp::{Filter};

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

    let routes = health_route
      .with(warp::cors().allow_any_origin())
      .recover(error::handle_rejection);

    warp::serve(routes).run(([127, 0, 0, 1], 8000)).await;
}

fn with_db(db_pool: PgPool) -> impl Filter<Extract = (PgPool,), Error = Infallible> + Clone {
    warp::any().map(move || db_pool.clone())
}
