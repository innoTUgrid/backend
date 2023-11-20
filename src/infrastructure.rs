use crate::handlers::{add_meta, add_timeseries, get_timeseries_by_identifier, ping, read_meta};
use axum::routing::post;
use axum::{routing::get, Router};
use dotenv::dotenv;
use sqlx::Pool;
use sqlx::Postgres;

pub async fn create_connection_pool() -> Pool<Postgres> {
    dotenv().ok();
    let database_url =
        std::env::var("DATABASE_URL").expect("Couldn't find database url in .env file");
    Pool::<Postgres>::connect(&database_url)
        .await
        .expect("Failed to create database connection pool")
}

pub fn create_router(pool: Pool<Postgres>) -> Router {
    let app = Router::new()
        .route("/", get(ping))
        .route("/v1/", get(ping))
        .route("/v1/meta/", post(add_meta))
        .route("/v1/meta/", get(read_meta))
        .route("/v1/ts/", post(add_timeseries))
        .route("/v1/ts/:identifier/", get(get_timeseries_by_identifier))
        .with_state(pool);
    app
}
