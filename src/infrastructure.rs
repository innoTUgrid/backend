use crate::handlers::{add_meta, add_timeseries, get_timeseries_by_identifier, ping, read_meta};
use axum::routing::post;
use axum::{routing::get, Router};
use axum::extract::DefaultBodyLimit;
use dotenv::dotenv;
use sqlx::Pool;
use sqlx::Postgres;

/*
// should run migrations but fails to do so
pub async fn run_migrations(pool: &Pool<Postgres>) -> Result<(), sqlx::Error> {
    sqlx::migrate!("./migrations").run(pool).await
}*/

pub async fn create_connection_pool() -> Pool<Postgres> {
    dotenv().ok();
    
    let database_url =
        std::env::var("DATABASE_URL").expect("Couldn't find database url in .env file");
    
    Pool::<Postgres>::connect(&database_url)
        .await
        .expect("Failed to create database connection pool")
}

pub fn create_router(pool: Pool<Postgres>) -> Router {
    
    Router::new()
        .route("/", get(ping))
        .route("/v1/", get(ping))
        .route("/v1/meta/", post(add_meta))
        .route("/v1/meta/", get(read_meta))
        .route("/v1/ts/", post(add_timeseries))
        .route("/v1/ts/:identifier/", get(get_timeseries_by_identifier))
        // limit file size to 10MB
        .layer(DefaultBodyLimit::max(1024*1024*10))
        .with_state(pool)
}
