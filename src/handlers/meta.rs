use crate::error::ApiError;

use crate::models::{MetaInput, MetaOutput, MetaRows, Pagination, Result};

use axum::extract::{Path, Query, State};
use axum::Json;
use axum_extra::extract::WithRejection;

use crate::cache::Cache;
use sqlx::{Pool, Postgres, Row};
use std::string::String;
use axum::http::Uri;

pub async fn read_meta(
    State(pool): State<Pool<Postgres>>,
    pagination: Query<Pagination>,
    uri: Uri,
) -> Result<Json<MetaRows>, ApiError> {
    let mut cache = Cache::new().await.unwrap();
    let key = format!("{}", uri);
    let cached = cache.get(&key).await;
    match cached {
        Ok(cached) => {
            let meta_rows: MetaRows = serde_json::from_str(&cached).unwrap();
            Ok(Json(meta_rows))
        }
        Err(_) => {
            let query_offset = pagination.get_offset();
            let mut meta_query = sqlx::query(
                r"
        select
            meta.id as id,
            meta.identifier as identifier,
            meta.unit as unit,
            meta.consumption as consumption,
            meta.description as description,
            energy_carrier.name as carrier,
            meta.local as local,
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
                    consumption: row.get(3),
                    description: row.get(4),
                    carrier: row.get(5),
                    local: row.get(6),
                    min_timestamp: row.get(7),
                    max_timestamp: row.get(8),
                };
                json_values.push(meta_value);
            }
            let meta_rows = MetaRows {
                values: json_values,
            };
            let serialized = serde_json::to_string(&meta_rows).unwrap();
            cache.set(&key, &serialized, 5 * 60).await.unwrap();
            Ok(Json(meta_rows))
        }
    }
}

pub async fn get_meta_by_identifier(
    State(pool): State<Pool<Postgres>>,
    Path(identifier): Path<String>,
) -> Result<Json<MetaOutput>, ApiError> {
    /* NOTE: using a compile time checked query va query_as! results in a nullability error for the carrier field. Might have something to do with https://github.com/launchbadge/sqlx/issues/1852 */
    let meta_output = sqlx::query_as::<_, MetaOutput>(
        r"
        select
            meta.id as id,
            meta.identifier as identifier,
            meta.unit as unit,
            meta.consumption as consumption,
            meta.description as description,
            energy_carrier.name as carrier,
            meta.local as local,
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
    )
    .bind(identifier)
    .fetch_one(&pool)
    .await?;
    Ok(Json(meta_output))
}

pub async fn add_meta(
    State(pool): State<Pool<Postgres>>,
    WithRejection(Json(meta), _): WithRejection<Json<MetaInput>, ApiError>,
) -> Result<Json<MetaOutput>, ApiError> {
    let meta_output: MetaOutput = sqlx::query_as!(
        MetaOutput,
        r"
        insert into meta (identifier, unit, carrier, consumption, description, local)
        select
            $1,
            $2,
            case
                when $3::text is not null then
                    (select energy_carrier.id from energy_carrier where energy_carrier.name = $3)
                else null
            end,
            $4,
            $5,
            $6
        returning
            id,
            identifier,
            unit,
            $3 as carrier,
            $4 as consumption,
            $5 as description,
            null::timestamptz as min_timestamp,
            null::timestamptz as max_timestamp,
            $6 as local",
        &meta.identifier,
        &meta.unit,
        meta.carrier.as_deref(),
        meta.consumption,
        meta.description,
        meta.local,
    )
    .fetch_one(&pool)
    .await?;

    Ok(Json(meta_output))
}
