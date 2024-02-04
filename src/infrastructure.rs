use crate::app_config::AppConfig;
use crate::error::ApiError;
use crate::handlers::config::{get_config, put_config};
use crate::handlers::kpi::{
    get_autarky, get_co2_savings, get_consumption, get_cost_savings, get_scope_one_emissions,
    get_scope_two_emissions, get_self_consumption, get_total_consumption,
};
use crate::handlers::meta::{add_meta, get_meta_by_identifier, read_meta};
use crate::handlers::timeseries::{
    add_timeseries, get_timeseries_by_identifier, resample_timeseries_by_identifier,
};
use crate::handlers::util::ping;
use crate::handlers::emission_factor::{add_emission_factor, get_emission_factor};
use crate::handlers::import::upload_timeseries;
use crate::models::Result;
use axum::extract::DefaultBodyLimit;
use axum::routing::post;
use axum::{routing::get, Router};
use dotenv::dotenv;
use sqlx::Postgres;
use sqlx::{ConnectOptions, Pool};
use std::str::FromStr;
use tower_http::cors::{Any, CorsLayer};
use tracing::log::LevelFilter;

pub async fn create_connection_pool(config: &AppConfig) -> Pool<Postgres> {
    dotenv().ok();
    // only log sql statements if log level is at least debug
    let postgres_connect_options = sqlx::postgres::PgConnectOptions::from_str(&config.database_url)
        .expect("Failed to parse database url")
        .log_statements(LevelFilter::Debug);
    Pool::<Postgres>::connect_with(postgres_connect_options)
        .await
        .expect("Failed to create database connection pool")
}

async fn fallback_handler() -> Result<ApiError> {
    Err(ApiError::NotFound)
}

pub fn create_router(pool: Pool<Postgres>) -> Router {
    // for swagger-ui and mitigating common errors for development
    let cors = CorsLayer::new().allow_origin(Any).allow_headers(Any);

    Router::new()
        .route("/", get(ping))
        .route("/v1/", get(ping))
        .route("/v1/config/", post(put_config))
        .route("/v1/config/", get(get_config))
        .route("/v1/kpi/consumption/", get(get_consumption))
        .route("/v1/kpi/total_consumption/", get(get_total_consumption))
        .route("/v1/kpi/scope_one_emissions/", get(get_scope_one_emissions))
        .route("/v1/kpi/scope_two_emissions/", get(get_scope_two_emissions))
        .route("/v1/kpi/self_consumption/", get(get_self_consumption))
        .route("/v1/kpi/autarky/", get(get_autarky))
        .route("/v1/kpi/cost_savings/", get(get_cost_savings))
        .route("/v1/kpi/co2_savings/", get(get_co2_savings))
        .route("/v1/meta/", post(add_meta))
        .route("/v1/meta/:identifier/", get(get_meta_by_identifier))
        .route("/v1/meta/", get(read_meta))
        .route("/v1/ts/", post(add_timeseries))
        .route("/v1/ts/upload/", post(upload_timeseries))
        .route("/v1/ts/:identifier/", get(get_timeseries_by_identifier))
        .route(
            "/v1/ts/:identifier/resample/",
            get(resample_timeseries_by_identifier),
        )
        .route("/v1/emission_factor/", get(get_emission_factor))
        .route("/v1/emission_factor/", post(add_emission_factor))
        .fallback(get(fallback_handler))
        .layer(cors)
        // limit file size to 10MB
        .with_state(pool)
        .layer(DefaultBodyLimit::max(1024 * 1024 * 10))
}

#[cfg(test)]
mod tests {
    use axum::http::header;
    use axum_test_helper::TestClient;

    use crate::{app_config::AppConfig, create_connection_pool};
    #[tokio::test]
    async fn test_create_pool_connection() {
        let pool = create_connection_pool(&AppConfig::new()).await;
        let row: (i32,) = sqlx::query_as("SELECT 1")
            .fetch_one(&pool)
            .await
            .expect("Failed to fetch from database");
        assert_eq!(row.0, 1, "Could not connect to database")
    }

    #[tokio::test]
    async fn test_cors() {
        let pool = create_connection_pool(&AppConfig::new()).await;
        let router = crate::create_router(pool);
        let client = TestClient::new(router);

        let response = client
            .get("/v1/")
            .header(header::ORIGIN, "https://example.com")
            .send()
            .await;

        assert_eq!(
            response
                .headers()
                .get(header::ACCESS_CONTROL_ALLOW_ORIGIN)
                .unwrap(),
            "*",
        );
    }
}
