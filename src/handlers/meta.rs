use crate::error::ApiError;

use crate::models::{MetaInput, MetaOutput, MetaRows, Pagination, Result};

use axum::extract::{Path, Query, State};
use axum::Json;
use axum_extra::extract::WithRejection;

use sqlx::{Pool, Postgres, Row};
use std::string::String;

pub async fn read_meta(
    State(pool): State<Pool<Postgres>>,
    pagination: Query<Pagination>,
) -> Result<Json<MetaRows>, ApiError> {
    let query_offset = pagination.get_offset();
    let mut meta_query = sqlx::query(
        r"
        select
            meta.id as id,
            meta.identifier as identifier,
            meta.unit as unit,
            energy_carrier.name as carrier,
            min(ts.series_timestamp) as min_timestamp,
            max(ts.series_timestamp) as max_timestamp
        from meta
            left join energy_carrier on meta.carrier = energy_carrier.id
            left join ts on meta.id = ts.meta_id
        group by
            meta.id,
            energy_carrier.name
        order by
            id
        offset $1
        limit $2",
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
            min_timestamp: row.get(4),
            max_timestamp: row.get(5),
        };
        json_values.push(meta_value);
    }
    let meta_rows = MetaRows {
        values: json_values,
    };
    Ok(Json(meta_rows))
}

pub async fn get_meta_by_identifier(
    State(pool): State<Pool<Postgres>>,
    Path(identifier): Path<String>,
) -> Result<Json<MetaOutput>, ApiError> {
    let maybe_meta = sqlx::query_as!(
        MetaOutput,
        r"
        select
            meta.id as id,
            meta.identifier as identifier,
            meta.unit as unit,
            energy_carrier.name as carrier,
            min(ts.series_timestamp) as min_timestamp,
            max(ts.series_timestamp) as max_timestamp
        from meta
            left join energy_carrier on meta.carrier = energy_carrier.id
            left join ts on meta.id = ts.meta_id
        where
            meta.identifier = $1
        group by
            meta.id,
            energy_carrier.name
        order by
            id
            ",
        identifier
    )
    .fetch_optional(&pool)
    .await?;
    match maybe_meta {
        Some(meta_output) => Ok(Json(meta_output)),
        None => Err(ApiError::NotFound),
    }
}

pub async fn add_meta(
    State(pool): State<Pool<Postgres>>,
    WithRejection(Json(meta), _): WithRejection<Json<MetaInput>, ApiError>,
) -> Result<Json<MetaOutput>, ApiError> {
    let meta_output: MetaOutput = sqlx::query_as!(
        MetaOutput,
        r"
        insert into meta (identifier, unit, carrier)
        select
            $1,
            $2,
            case
                when $3::text is not null then
                    (select energy_carrier.id from energy_carrier where energy_carrier.name = $3)
                else null
            end
        returning
            id,
            identifier,
            unit,
            $3 as carrier,
            null::timestamptz as min_timestamp,
            null::timestamptz as max_timestamp",
        &meta.identifier,
        &meta.unit,
        meta.carrier.as_deref(),
    )
    .fetch_one(&pool)
    .await?;

    Ok(Json(meta_output))
}
