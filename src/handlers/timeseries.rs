use crate::models::TimeseriesMeta;
use crate::models::{Datapoint, ResampledDatapoint, ResampledTimeseries, Resampling, Result};
use crate::models::{NewDatapoint, TimeseriesBody};
use crate::models::{Timeseries, TimestampFilter};

use axum::extract::{Path, Query, State};
use axum::Json;

use sqlx::{Pool, Postgres};
use std::string::String;

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
        r#"
        select meta.id as id, identifier, unit, energy_carrier.name as carrier, consumption
        from meta join energy_carrier on meta.carrier = energy_carrier.id
        where meta.identifier = $1"#,
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
        r#"
        select meta.id as id, identifier, unit, energy_carrier.name as carrier, consumption
        from meta join energy_carrier on meta.carrier = energy_carrier.id
        where meta.identifier = $1"#,
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

/// fetch all timseries matching a comma-seperated list of identifiers
/**/
pub async fn add_timeseries(
    State(pool): State<Pool<Postgres>>,
    req: Json<TimeseriesBody<NewDatapoint>>,
) -> Result<Json<TimeseriesBody<Datapoint>>> {
    let metadata = sqlx::query_as!(
        TimeseriesMeta,
        r#"
        select meta.id as id, identifier, unit, energy_carrier.name as carrier, consumption
        from meta join energy_carrier on meta.carrier = energy_carrier.id
        where meta.identifier = $1"#,
        req.timeseries.identifier,
    )
    .fetch_one(&pool)
    .await?;

    let timeseries = sqlx::query_as!(
        Datapoint,
        r#"
        insert into ts (series_timestamp, series_value, meta_id)
        values ($1, $2, $3)
        returning id, series_timestamp as timestamp, series_value as value, created_at, updated_at
        "#,
        req.timeseries.timestamp,
        req.timeseries.value,
        metadata.id
    )
    .fetch_one(&pool)
    .await?;

    Ok(Json(TimeseriesBody { timeseries }))
}
