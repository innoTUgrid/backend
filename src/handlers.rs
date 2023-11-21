use crate::models::Timeseries;
use crate::models::TimeseriesBody;
use crate::models::TimeseriesMeta;
use crate::models::TimeseriesNew;
use crate::models::TimeseriesWithoutMetadata;
use crate::models::{
    Datapoint, MetaInput, MetaOutput, MetaRows, Pagination, PingResponse, ResampledDatapoint,
    ResampledTimeseries, Resampling, Result, TimeseriesWithMetadata,
};
use axum::extract::{Path, Query, State};
use axum::Json;
use axum_extra::extract::WithRejection;
use sqlx::{Pool, Postgres, Row};

use crate::{
    error::ApiError,
};

/// timeseries values for specific metadata and a given interval
pub async fn resample_timeseries_by_identifier(
    State(pool): State<Pool<Postgres>>,
    Path(identifier): Path<String>,
    Query(resampling): Query<Resampling>,
) -> Result<Json<ResampledTimeseries>> {
    let pg_resampling_interval = resampling.map_interval()?;
    let metadata = sqlx::query_as!(
        TimeseriesMeta,
        r#"select id, identifier, unit, carrier, consumption from meta where meta.identifier = $1"#,
        identifier,
    )
    .fetch_one(&pool)
    .await?;

    let datapoints = sqlx::query_as!(
        ResampledDatapoint,
        r#"
        select
            time_bucket($2::interval, ts.series_timestamp) as timestamp,
            avg(ts.series_value) as mean_value 
        from ts
        where ts.meta_id = $1
        group by $1, series_timestamp
        order by series_timestamp
        "#,
        metadata.id,
        pg_resampling_interval
    )
    .fetch_all(&pool)
    .await?;

    let response = ResampledTimeseries {
        datapoints,
        meta: metadata,
    };
    Ok(Json(response))
}

/// Get all timeseries values for specific metadata
pub async fn get_timeseries_by_identifier(
    State(pool): State<Pool<Postgres>>,
    Path(identifier): Path<String>,
) -> Result<Json<Timeseries>> {
    let mut rows = sqlx::query_as!(
        TimeseriesWithMetadata,
        r#"
        select
            meta.id meta_id,
            meta.identifier identifier,
            meta.unit unit,
            meta.carrier carrier,
            meta.consumption consumption,
            ts.id series_id,
            ts.series_timestamp,
            ts.series_value,
            ts.created_at created_at,
            ts.updated_at updated_at
        from ts, meta
        where ts.meta_id = meta.id
        and meta.identifier = $1
        order by ts.series_timestamp
        "#,
        identifier
    )
    .fetch_all(&pool)
    .await?;

    let datapoints = rows
        .iter()
        .map(|row| Datapoint {
            id: row.series_id,
            timestamp: row.series_timestamp,
            value: row.series_value,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
        .collect();
    let first_row = rows.remove(0);
    let metadata = TimeseriesMeta {
        id: first_row.meta_id,
        identifier: first_row.identifier,
        unit: first_row.unit,
        carrier: first_row.carrier,
        consumption: first_row.consumption,
    };

    let response = Timeseries {
        datapoints,
        meta: metadata,
    };

    Ok(Json(response))
}

pub async fn add_timeseries(
    State(pool): State<Pool<Postgres>>,
    req: Json<TimeseriesBody<TimeseriesNew>>,
) -> Result<Json<TimeseriesBody<TimeseriesWithoutMetadata>>> {
    let metadata = sqlx::query_as!(
        TimeseriesMeta,
        r#"select id, identifier, unit, carrier, consumption from meta where meta.identifier = $1"#,
        req.timeseries.identifier,
    )
    .fetch_one(&pool)
    .await?;
    let timeseries = sqlx::query_as!(
        TimeseriesWithoutMetadata,
        r#"
        insert into ts (series_timestamp, series_value, meta_id)
        values ($1, $2, $3)
        returning id, series_timestamp, series_value, created_at, updated_at
        "#,
        req.timeseries.series_timestamp,
        req.timeseries.series_value,
        metadata.id
    )
    .fetch_one(&pool)
    .await?;
    Ok(Json(TimeseriesBody {
        timeseries,
    }))
}

pub async fn read_meta(
    State(pool): State<Pool<Postgres>>,
    pagination: Query<Pagination>,
) -> Result<Json<MetaRows>, ApiError> {
    let query_offset =
        pagination.0.page.unwrap_or_default() * pagination.0.per_page.unwrap_or_default();
    let mut meta_query = sqlx::query(
        "select id, identifier, unit, carrier from meta order by id offset $1 limit $2",
    );
    meta_query = meta_query.bind(query_offset);
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
        values: json_values,
    };
    Ok(Json(meta_rows))
}

pub async fn ping(State(_pool): State<Pool<Postgres>>) -> Json<PingResponse> {
    Json(PingResponse::default())
}

pub async fn add_meta(
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

#[cfg(test)]

mod tests {
    use super::*;
    use crate::infrastructure::create_router;
    use crate::infrastructure::create_connection_pool;
    use axum_test_helper::TestClient;
    use serde_json::json;
    use time::OffsetDateTime;

    #[tokio::test]
    async fn test_add_timeseries() {
        let pool = create_connection_pool().await;
        let router = create_router(pool);

        let client = TestClient::new(router);

        let timeseries = TimeseriesNew {
            series_timestamp: OffsetDateTime::now_utc(),
            series_value: 42.0,
            identifier: String::from("testIdentifier"),
        };
        let res = client
            .post("/v1/ts/")
            .json(&TimeseriesBody { timeseries })
            .send()
            .await;
        assert!(res.status().is_success());
    }

    #[tokio::test]
    async fn test_add_timeseries_bad_data() {
        let pool = create_connection_pool().await;
        let router = create_router(pool);
        let client = TestClient::new(router);

        let rfc_3339_format = &time::format_description::well_known::Rfc3339; 
        let timeseries = json!({
            "series_timestamp": OffsetDateTime::now_utc().format(rfc_3339_format).unwrap(),
            "series_value": 42,
            "wrongKey": String::from("testIdentifier"),
        });
        let response = client
            .post("/v1/ts/")
            .json(&TimeseriesBody { timeseries })
            .send()
            .await;
        assert!(response.status().is_client_error());
    }

    #[tokio::test]
    async fn test_add_meta() {
        let pool = create_connection_pool().await;
        let router = create_router(pool);

        let client = TestClient::new(router);

        let meta = MetaInput {
            identifier: String::from("testIdentifier"),
            unit: String::from("testUnit"),
            carrier: Some(String::from("testCarrier")),
        };
        let res = client.post("/v1/meta/").json(&meta).send().await;
        assert!(res.status().is_success());
    }

    #[tokio::test]
    async fn test_add_meta_bad_data() {
        let pool = create_connection_pool().await;
        let router = create_router(pool);
        let client = TestClient::new(router);

        let bad_metadata = json!(
            {
                "identifier": "testIdentifier",
                "unitSchmunit": "testUnit",
                "carrier": 42
            }
        );
        let response = client.post("/v1/meta/").json(&bad_metadata).send().await;
        assert!(response.status().is_client_error());
    }

    #[tokio::test]
    async fn test_read_meta() {

    }

    #[tokio::test]
    async fn test_ping() {
        // Setup
        let pool = create_connection_pool().await;
        let app = create_router(pool.clone());
        let client = TestClient::new(app);
    
        // Send a request to the ping endpoint
        let response = client.get("/v1/").send().await;
    
        // Verify the response
        assert!(response.status().is_success());
        let body: PingResponse = response.json().await;
        assert_eq!(body.message, "0xDECAFBAD");
    }

}

