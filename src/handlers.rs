use crate::models::Result;
use crate::models::Timeseries;
use crate::models::TimeseriesBody;
use crate::models::TimeseriesFlat;
use crate::models::TimeseriesFromQuery;
use crate::models::TimeseriesNew;
use crate::models::{TimeseriesMeta, TimeseriesResponse, Timestamptz};
use axum::extract::{Path, State};
use axum::Json;
use futures::TryStreamExt;
use sqlx::{Pool, Postgres};


/// Get all timeseries values for specific metadata
pub async fn get_timeseries_by_identifier(
    State(pool): State<Pool<Postgres>>,
    Path(identifier): Path<String>,
) -> Result<Json<TimeseriesResponse>> {
    let timeseries = sqlx::query_as!(
        TimeseriesFromQuery,
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
    .fetch(&pool)
    .map_ok(TimeseriesFromQuery::into_timeseries)
    .try_collect()
    .await?;
    Ok(Json(TimeseriesResponse { data: timeseries }))
}

pub async fn add_timeseries(
    State(pool): State<Pool<Postgres>>,
    req: Json<TimeseriesBody<TimeseriesNew>>,
) -> Result<Json<TimeseriesBody<Timeseries>>> {
    let metadata = sqlx::query_as!(
        TimeseriesMeta,
        r#"select id, identifier, unit, carrier, consumption from meta where meta.identifier = $1"#,
        req.timeseries.identifier,
    )
    .fetch_one(&pool)
    .await?;
    let timeseries = sqlx::query_as!(
        TimeseriesFlat,
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
        timeseries: Timeseries {
            id: timeseries.id,
            series_timestamp: Timestamptz(timeseries.series_timestamp),
            series_value: timeseries.series_value,
            created_at: Timestamptz(timeseries.created_at),
            updated_at: Timestamptz(timeseries.updated_at),
            meta: TimeseriesMeta {
                id: metadata.id,
                identifier: metadata.identifier,
                unit: metadata.unit,
                carrier: metadata.carrier,
                consumption: metadata.consumption,
            },
        },
    }))
}
