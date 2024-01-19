use crate::error::ApiError;

use crate::models::{MetaInput, MetaOutput, MetaRows, Pagination, Result};

use axum::extract::{Query, State};
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
        select meta.id as id, meta.identifier as identifier, meta.unit as unit, energy_carrier.name as carrier
        from meta join energy_carrier on meta.carrier = energy_carrier.id
        order by id offset $1 limit $2",
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

pub async fn add_meta(
    State(pool): State<Pool<Postgres>>,
    WithRejection(Json(meta), _): WithRejection<Json<MetaInput>, ApiError>,
) -> Result<Json<MetaOutput>, ApiError> {
    let meta_output: MetaOutput = sqlx::query_as!(
        MetaOutput,
        r"
        insert into meta (identifier, unit, carrier)
        select $1, $2,
            case
                when $3::text is not null then (select energy_carrier.id from energy_carrier where energy_carrier.name = $3)
                else null
            end
        returning id, identifier, unit, $3 as carrier",
        &meta.identifier,
        &meta.unit,
        meta.carrier.as_deref(),
    )
    .fetch_one(&pool)
    .await?;

    Ok(Json(meta_output))
}
