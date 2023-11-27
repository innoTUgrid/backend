use crate::handlers::{add_meta, add_timeseries, get_timeseries_by_identifier, ping, read_meta};
use axum::extract::DefaultBodyLimit;
use axum::routing::post;
use axum::{routing::get, Router};
use dotenv::dotenv;
use sqlx::{ConnectOptions, Pool};
use sqlx::Postgres;
use tracing::log::LevelFilter;
use std::str::FromStr;

pub async fn create_connection_pool() -> Pool<Postgres> {
    dotenv().ok();
    let database_url =
        std::env::var("DATABASE_URL").expect("Couldn't find database url in .env file");
    let postgres_connect_options = sqlx::postgres::PgConnectOptions::from_str(&database_url)
        .expect("Failed to parse database url")
        .log_statements(LevelFilter::Debug);
    Pool::<Postgres>::connect_with(postgres_connect_options)
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
        .layer(DefaultBodyLimit::max(1024 * 1024 * 10))
        .with_state(pool)
}
