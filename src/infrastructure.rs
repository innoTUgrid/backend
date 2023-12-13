use crate::handlers::{
    add_meta, add_timeseries, get_autarky, get_scope_two_emissions, get_self_consumption,
    get_timeseries_by_identifier, ping, read_meta, resample_timeseries_by_identifier,
    upload_timeseries,
};
use axum::extract::DefaultBodyLimit;
use axum::routing::post;
use axum::{routing::get, Router};
use dotenv::dotenv;
use sqlx::Postgres;
use sqlx::{ConnectOptions, Pool};
use std::str::FromStr;
use tracing::log::LevelFilter;
use tracing::Level;

pub async fn create_connection_pool() -> Pool<Postgres> {
    dotenv().ok();
    let database_url =
        std::env::var("DATABASE_URL").expect("Couldn't find database url in .env file");
    // only log sql statements if log level is at least debug
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
        .route("/v1/kpi/self_consumption/", get(get_self_consumption))
        .route("/v1/kpi/autarky/", get(get_autarky))
        .route("/v1/kpi/scope_two_emissions/", get(get_scope_two_emissions))
        .route("/v1/meta/", post(add_meta))
        .route("/v1/meta/", get(read_meta))
        .route("/v1/ts/", post(add_timeseries))
        .route("/v1/ts/upload", post(upload_timeseries))
        .route("/v1/ts/:identifier/", get(get_timeseries_by_identifier))
        .route(
            "/v1/ts/:identifier/resample",
            get(resample_timeseries_by_identifier),
        )
        // limit file size to 10MB
        .with_state(pool)
        .layer(DefaultBodyLimit::max(1024 * 1024 * 10))
}

pub fn read_log_level() -> Level {
    dotenv().ok();
    let log_level = std::env::var("LOG_LEVEL").unwrap_or_else(|_| "INFO".to_string());
    let log_level = match log_level.as_str() {
        "DEBUG" => Level::DEBUG,
        "INFO" => Level::INFO,
        "WARN" => Level::WARN,
        "ERROR" => Level::ERROR,
        "TRACE" => Level::TRACE,
        _ => Level::INFO,
    };
    log_level
}

#[cfg(test)]
mod tests {
    use crate::create_connection_pool;
    #[tokio::test]
    async fn test_create_pool_connection() {
        let pool = create_connection_pool().await;
        let row: (i32,) = sqlx::query_as("SELECT 1")
            .fetch_one(&pool)
            .await
            .expect("Failed to fetch from database");
        assert_eq!(row.0, 1, "Could not connect to database")
    }
}
