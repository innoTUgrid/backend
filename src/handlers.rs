use crate::error::ApiError;
use crate::models::TimeseriesBody;
use crate::models::TimeseriesMeta;
use crate::models::TimeseriesNew;
use crate::models::TimeseriesWithoutMetadata;
use crate::models::{
    Datapoint, MetaInput, MetaOutput, MetaRows, Pagination, PingResponse, ResampledDatapoint,
    ResampledTimeseries, Resampling, Result,
};
use crate::models::{Timeseries, TimestampFilter};
use axum::extract::Multipart;
use axum::extract::{Path, Query, State};
use axum::Json;
use axum_extra::extract::WithRejection;
use sqlx::{Pool, Postgres, Row};

use std::string::String;
use time::{Duration, OffsetDateTime};
use time::format_description::well_known::Rfc3339;
//use csv_async::{AsyncReaderBuilder, AsyncReader, AsyncDeserializer, ByteRecord, StringRecord};
use serde::Deserialize;

use crate::{
    error::ApiError,
};

/// timeseries values for specific metadata and a given interval
pub async fn resample_timeseries_by_identifier(
    State(pool): State<Pool<Postgres>>,
    Path(identifier): Path<String>,
    Query(resampling): Query<Resampling>,
    Query(timestamp_filter): Query<TimestampFilter>,
) -> Result<Json<ResampledTimeseries>> {
    let pg_resampling_interval = resampling.map_interval()?;
    let metadata = sqlx::query_as!(
        TimeseriesMeta,
        r#"select id, identifier, unit, carrier, consumption from meta where meta.identifier = $1"#,
        identifier,
    )
    .fetch_one(&pool)
    .await?;

    let timestamp_from = timestamp_filter.from.unwrap();
    let timestamp_to = timestamp_filter.to.unwrap();

    let datapoints = sqlx::query_as!(
        ResampledDatapoint,
        r#"
        select
            time_bucket($2::interval, ts.series_timestamp) as bucket,
            avg(ts.series_value) as mean_value 
        from ts
        where ts.meta_id = $1
        and ts.series_timestamp >= $3
        and ts.series_timestamp <= $4
        group by bucket
        order by bucket
        "#,
        metadata.id,
        pg_resampling_interval,
        timestamp_from,
        timestamp_to
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
    Query(timestamp_filter): Query<TimestampFilter>,
) -> Result<Json<Timeseries>> {
    let from_timestamp = timestamp_filter.from.unwrap();
    let to_timestamp = timestamp_filter.to.unwrap();
    // we do the join in the backend here
    // this hits the database twice, but we avoid a branch and can simplify the code
    // additionally we can always return matching metadata even if query param filters lead to empty result set
    let metadata = sqlx::query_as!(
        TimeseriesMeta,
        r#"select id, identifier, unit, carrier, consumption from meta where meta.identifier = $1"#,
        identifier,
    )
    .fetch_one(&pool)
    .await?;
    let rows = sqlx::query_as!(
        Datapoint,
        r#"
        select
            ts.id,
            ts.series_timestamp as "timestamp",
            ts.series_value as "value",
            ts.created_at created_at,
            ts.updated_at updated_at
        from ts
        where ts.meta_id = $1
        and ts.series_timestamp >= $2
        and ts.series_timestamp <= $3
        "#,
        metadata.id,
        from_timestamp,
        to_timestamp,
    )
    .fetch_all(&pool)
    .await?;
    let response = Timeseries {
        datapoints: rows,
        meta: metadata,
    };
    Ok(Json(response))
}

/**/
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

    Ok(Json(TimeseriesBody { timeseries }))
}

/*
upload a file from a form and bulk insert it into the database
docs: https://docs.rs/axum/latest/axum/extract/multipart/struct.Field.html
test: curl -F upload=@/home/ole/poetry/jupyter_lab/jupyter_lab/DSP/dataset/inno2grid_backend_test.csv 127.0.0.1:3000/v1/ts/upload
*/
pub async fn upload_timeseries(
    State(_pool): State<Pool<Postgres>>,
    mut multipart: Multipart,
) -> Result<Json<String>, ApiError> {

    let mut data_string = String::new();

    // iterate over the fields of the form data
    while let Some(field) = multipart.next_field().await? {
        let field_name = field.name().unwrap_or("Unnamed field").to_string();
        println!("field_name: `{}`", field_name);
        if let Some(file_name) = field.file_name() {
            println!("file_name: `{}`", file_name);
            if file_name.ends_with(".csv") {
                
                // convert to: <std::string::String>
                data_string = field.text().await?;
                println!("data_string: {:?}", data_string);

                let rows: Vec<Vec<String>> = data_string
                .lines() // Split into lines
                .map(|line| line.split(',') // Split each line by comma
                    .map(|s| s.to_string()) // Convert each &str to String
                    .collect()) // Collect into a Vec<String>
                .collect(); // Collect all the Vec<String> into a Vec<Vec<String>>
                println!("rows: {:?}", rows);

                println!("headers: {:?}", rows[0]);
                
                /*
                for row in rows.iter().skip(1) {
                    println!("row: {:?}", row);
                    let query = sqlx::query!(
                        "INSERT INTO ts (series_timestamp, series_value, meta_id)
                        VALUES ($1, $2, $3)",
                        OffsetDateTime::parse(&row[0], &Rfc3339)?, 
                        row[1].parse::<f64>()?, 
                        row[2].parse::<i32>()?
                    );
                    query.execute(&pool).await?;
                }*/
                /*
                for row in rows.iter().skip(1) {
                    sqlx::query_as!(
                        TimeseriesWithoutMetadata,
                        r#"
                        insert into ts (series_timestamp, series_value, meta_id)
                        values ($1, $2, $3)
                        returning id, series_timestamp, series_value, created_at, updated_at
                        "#,
                        OffsetDateTime::parse(&row[0], &Rfc3339)?, 
                        row[1].parse::<f64>()?, 
                        row[2].parse::<i32>()?
                    )
                    .fetch_one(&pool)
                    .await?;
                }*/
            }
        }
    }

    //Ok(Json("File uploaded successfully".to_string()))
    Ok(Json(data_string))
}

pub async fn read_meta(
    State(pool): State<Pool<Postgres>>,
    pagination: Query<Pagination>,
) -> Result<Json<MetaRows>, ApiError> {
    let query_offset = pagination.get_offset();
    let mut meta_query = sqlx::query(
        "select id, identifier, unit, carrier from meta order by id offset $1 limit $2",
    );
    meta_query = meta_query.bind(query_offset);
    meta_query = meta_query.bind(pagination.get_per_page_or_default());
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
    let meta_output: MetaOutput = sqlx::query_as!(
        MetaOutput,
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
    use crate::infrastructure::create_connection_pool;
    use crate::infrastructure::create_router;
    use axum_test_helper::TestClient;
    use rand::distributions::{Alphanumeric, DistString};
    use serde_json::json;
    use time::OffsetDateTime;

    fn get_random_string(size: usize) -> String {
        Alphanumeric.sample_string(&mut rand::thread_rng(), size)
    }

    async fn get_client() -> TestClient {
        let pool = create_connection_pool().await;
        let router = create_router(pool);

        TestClient::new(router)
    }

    async fn add_meta(client: &TestClient, identifier: &str) -> MetaOutput {
        let meta = MetaInput {
            identifier: identifier.to_string(),
            unit: String::from("testUnit"),
            carrier: Some(String::from("testCarrier")),
        };
        let res = client.post("/v1/meta/").json(&meta).send().await;
        assert!(res.status().is_success());

        let r: MetaOutput = res.json().await;
        assert_eq!(r.identifier, identifier);
        r
    }

    async fn add_timeseries(
        client: &TestClient,
        identifier: &str,
        value: f64,
    ) -> TimeseriesBody<TimeseriesWithoutMetadata> {
        let timeseries = TimeseriesNew {
            series_timestamp: OffsetDateTime::now_utc(),
            series_value: value,
            identifier: identifier.to_string(),
        };
        let res = client
            .post("/v1/ts/")
            .json(&TimeseriesBody { timeseries })
            .send()
            .await;
        assert!(res.status().is_success());

        let r: TimeseriesBody<TimeseriesWithoutMetadata> = res.json().await;
        assert_eq!(r.timeseries.series_value, value);
        r
    }

    #[tokio::test]
    async fn test_add_timeseries_bad_data() {
        let client = get_client().await;
        let identifier = get_random_string(10);
        add_meta(&client, &identifier).await;

        let rfc_3339_format = &time::format_description::well_known::Rfc3339;
        let timeseries = json!({
            "series_timestamp": OffsetDateTime::now_utc().format(rfc_3339_format).unwrap(),
            "series_value": 42,
            "wrongKey": identifier.to_string(),
        });
        let response = client
            .post("/v1/ts/")
            .json(&TimeseriesBody { timeseries })
            .send()
            .await;
        assert!(response.status().is_client_error());
    }

    #[tokio::test]
    async fn test_add_meta_bad_data() {
        let client = get_client().await;

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
        let client = get_client().await;

        let identifier = get_random_string(10);
        let meta = add_meta(&client, &identifier).await;

        let response = client.get("/v1/meta/").send().await;
        assert!(response.status().is_success());

        let body: MetaRows = response.json().await;

        assert!(
            body.values.iter().any(|x| x.identifier == meta.identifier),
            "identifier not found in response"
        );
    }

    #[tokio::test]
    async fn test_ping() {
        // Setup
        let client = get_client().await;

        // Send a request to the ping endpoint
        let response = client.get("/v1/").send().await;

        // Verify the response
        assert!(response.status().is_success());
        let body: PingResponse = response.json().await;
        assert_eq!(body.message, "0xDECAFBAD");
    }

    #[tokio::test]
    async fn test_add_timeseries() {
        let client = get_client().await;
        let identifier = get_random_string(10);

        add_meta(&client, &identifier).await;
        add_timeseries(&client, &identifier, 42.0).await;
    }

    #[tokio::test]
    async fn test_get_timeseries_by_identifier() {
        let client = get_client().await;
        let identifier = get_random_string(10);

        add_meta(&client, &identifier).await;
        add_timeseries(&client, &identifier, 42.0).await;

        let response = client.get(&format!("/v1/ts/{}/", identifier)).send().await;
        assert!(response.status().is_success());

        let body: Timeseries = response.json().await;
        assert_eq!(body.meta.identifier, identifier);
        assert_eq!(body.datapoints.len(), 1);
    }

    #[tokio::test]
    async fn test_get_timeseries_by_identifier_from_filter() {
        let client = get_client().await;
        let identifier = get_random_string(10);

        add_meta(&client, &identifier).await;
        add_timeseries(&client, &identifier, 42.0).await;

        let response = client
            .get(&format!("/v1/ts/{}/?from=2022-11-29T09:31:51Z", identifier))
            .send()
            .await;
        assert!(response.status().is_success());
        let body: Timeseries = response.json().await;
        assert_eq!(body.meta.identifier, identifier);
        assert_eq!(body.datapoints.len(), 1);
    }

    #[tokio::test]
    async fn test_get_timeseries_by_identifier_to_filter() {
        let client = get_client().await;
        let identifier = get_random_string(10);

        add_meta(&client, &identifier).await;
        add_timeseries(&client, &identifier, 42.0).await;

        let response = client
            .get(&format!("/v1/ts/{}/?to=2022-11-29T09:31:51Z", identifier))
            .send()
            .await;
        assert!(response.status().is_success());
        let body: Timeseries = response.json().await;
        assert_eq!(body.meta.identifier, identifier);
        assert_eq!(body.datapoints.len(), 0);
    }

    #[tokio::test]
    async fn test_get_timeseries_by_identifier_with_ts_filter() {
        let client = get_client().await;
        let identifier = get_random_string(10);

        add_meta(&client, &identifier).await;
        add_timeseries(&client, &identifier, 42.0).await;
        let response = client
            .get(&format!(
                "/v1/ts/{}/?from=2022-11-29T09:31:51Z&to=2022-12-01T00:00:00Z",
                identifier
            ))
            .send()
            .await;
        assert!(response.status().is_success());
        let body: Timeseries = response.json().await;
        assert_eq!(body.meta.identifier, identifier);
        assert_eq!(body.datapoints.len(), 0);
    }

    #[tokio::test]
    async fn test_resample_timeseries_by_identifier() {
        let client = get_client().await;
        let identifier = get_random_string(10);

        add_meta(&client, &identifier).await;
        add_timeseries(&client, &identifier, 42.0).await;
        add_timeseries(&client, &identifier, 66.0).await;

        let response = client
            .get(&format!("/v1/ts/{}/resample?interval=1hour", identifier))
            .send()
            .await;
        assert!(response.status().is_success());

        let body: ResampledTimeseries = response.json().await;
        assert_eq!(body.datapoints.first().unwrap().mean_value.unwrap(), 54.0);
    }
    #[tokio::test]
    async fn test_resample_timeseries_by_identifier_with_ts_filter_from() {
        let client = get_client().await;
        let identifier = get_random_string(10);

        add_meta(&client, &identifier).await;
        add_timeseries(&client, &identifier, 42.0).await;
        add_timeseries(&client, &identifier, 66.0).await;

        let response = client
            .get(&format!(
                "/v1/ts/{}/resample?interval=1hour&from=2022-11-29T09:31:51Z",
                identifier
            ))
            .send()
            .await;
        assert!(response.status().is_success());

        let body: ResampledTimeseries = response.json().await;
        assert_eq!(body.datapoints.first().unwrap().mean_value.unwrap(), 54.0);
    }
    #[tokio::test]
    async fn test_resample_timeseries_by_identifier_with_ts_filter_to() {
        let client = get_client().await;
        let identifier = get_random_string(10);

        add_meta(&client, &identifier).await;
        add_timeseries(&client, &identifier, 42.0).await;
        add_timeseries(&client, &identifier, 66.0).await;

        let response = client
            .get(&format!(
                "/v1/ts/{}/resample?interval=1hour&to=2022-11-29T09:31:51Z",
                identifier
            ))
            .send()
            .await;
        assert!(response.status().is_success());

        let body: ResampledTimeseries = response.json().await;
        assert_eq!(body.datapoints.len(), 0)
    }
    #[tokio::test]
    async fn test_resample_timeseries_by_identifier_with_ts_filter() {
        let client = get_client().await;
        let identifier = get_random_string(10);

        add_meta(&client, &identifier).await;
        add_timeseries(&client, &identifier, 42.0).await;
        add_timeseries(&client, &identifier, 66.0).await;

        let response = client
            .get(&format!("/v1/ts/{}/resample?interval=1hour&from=2022-11-29T09:31:51Z&to=2022-12-01T00:00:00Z", identifier))
            .send()
            .await;
        assert!(response.status().is_success());

        let body: ResampledTimeseries = response.json().await;
        assert_eq!(body.datapoints.len(), 0)
    }
    #[tokio::test]
    async fn test_resample_timeseries_by_identifier_bad_ts_filter() {
        let client = get_client().await;
        let identifier = get_random_string(10);

        add_meta(&client, &identifier).await;
        add_timeseries(&client, &identifier, 42.0).await;
        add_timeseries(&client, &identifier, 66.0).await;

        let response = client
            .get(&format!(
                "/v1/ts/{}/resample?interval=1hour&from=23542365346747&to=2022-12-01T00:00:00Z",
                identifier
            ))
            .send()
            .await;
        assert!(response.status().is_client_error());
    }
}
