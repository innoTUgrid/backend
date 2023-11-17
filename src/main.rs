mod error;
mod models;

mod handlers;

use axum::extract::{Json, Query, State};
use axum::routing::post;
use axum::{routing::get, Router};
use axum_extra::extract::WithRejection;
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use sqlx::Postgres;
use sqlx::{Pool, Row};

use crate::error::ApiError;
use crate::handlers::{add_timeseries, get_timeseries_by_identifier};
use crate::models::PingResponse;

async fn create_connection_pool() -> Pool<Postgres> {
    dotenv().ok();
    let database_url =
        std::env::var("DATABASE_URL").expect("Couldn't find database url in .env file");
    Pool::<Postgres>::connect(&database_url)
        .await
        .expect("Failed to create database connection pool")
}

#[derive(Deserialize)]
struct MetaInput {
    identifier: String,
    unit: String,
    carrier: Option<String>,
}
#[derive(Serialize)]
struct MetaOutput {
    id: i32,
    identifier: String,
    unit: String,
    carrier: Option<String>,
}

#[derive(Serialize)]
struct MetaRows {
    length: i32,
    values: Vec<MetaOutput>,
}

#[derive(Debug, Deserialize)]
struct Pagination {
    page: Option<i32>,
    per_page: Option<i32>,
}

impl Default for Pagination {
    fn default() -> Self {
        Self {
            page: Option::from(1),
            per_page: Option::from(1000),
        }
    }
}

async fn create_meta(
    State(pool): State<Pool<Postgres>>,
    WithRejection(Json(meta), _): WithRejection<Json<MetaInput>, ApiError>,
) -> Result<Json<MetaOutput>, ApiError> {
    let meta_output: MetaOutput = sqlx::query_as!(MetaOutput,
    "insert into meta (identifier, unit, carrier) values ($1, $2, $3) returning id, identifier, unit, carrier",
    &meta.identifier,
    &meta.unit,
    meta.carrier.as_deref(),
    )
    .fetch_one(&pool)
    .await?;
    Ok(Json(meta_output))
}

async fn read_meta(
    State(pool): State<Pool<Postgres>>,
    pagination: Query<Pagination>,
) -> Result<Json<MetaRows>, ApiError> {
    let query_offset =
        pagination.0.page.unwrap_or_default() * pagination.0.per_page.unwrap_or_default();
    let mut meta_query = sqlx::query(
        "select id, identifier, unit, carrier from meta order by id offset $1 limit $2",
    );
    meta_query = meta_query.bind(&query_offset);
    meta_query = meta_query.bind(pagination.0.per_page.unwrap_or_default());
    let meta_rows = meta_query.fetch_all(&pool).await?;
    let mut json_values: Vec<MetaOutput> = vec![];
    for row in &meta_rows {
        let meta_value = MetaOutput {
            id: row.get(0),
            identifier: row.get(1),
            unit: row.get(2),
            carrier: row.get(3),
        };
        json_values.push(meta_value);
    }
    let meta_rows = MetaRows {
        length: json_values.len() as i32,
        values: json_values,
    };
    println!("{:?}", pagination);
    Ok(Json(meta_rows))
}

async fn ping(State(pool): State<Pool<Postgres>>) -> Json<PingResponse> {
    Json(PingResponse::default())
}

#[tokio::main]
async fn main() {
    let _pool = create_connection_pool().await;

    let app = Router::new()
        .route("/", get(ping))
        .route("/v1/", get(ping))
        .route("/v1/meta/", post(create_meta))
        .route("/v1/meta/", get(read_meta))
        .route("/v1/ts/", post(add_timeseries))
        .route("/v1/ts/:identifier/", get(get_timeseries_by_identifier))
        .with_state(_pool);

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();

    println!("listening on 0.0.0.0:3000");
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
