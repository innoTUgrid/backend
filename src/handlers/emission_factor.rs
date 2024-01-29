use crate::models::Result;
use crate::{error::ApiError, models::EmissionFactor};
use axum::extract::State;
use sqlx::{Pool, Postgres};

use axum::Json;

pub async fn get_emission_factors(
    State(pool): State<Pool<Postgres>>,
) -> Result<Json<Vec<EmissionFactor>>, ApiError> {
    let emission_factors = sqlx::query_as!(
      EmissionFactor,
      r#"
        SELECT emission_factor.id, energy_carrier.name as carrier, factor, unit, source, source_url from emission_factor
        JOIN energy_carrier ON emission_factor.carrier = energy_carrier.id
        "#,
    )
    .fetch_all(&pool)
    .await?;

    Ok(Json(emission_factors))
}
