use crate::error::ApiError;
use crate::infrastructure::AppState;
use crate::models::{
    CreateEmissionFactorRequest, CreateEmissionFactorResponse, EmissionFactor, EmissionFactorFilter,
};
use axum::extract::{Query, State};
use axum::Json;

pub async fn add_emission_factor(
    State(app_state): State<AppState>,
    request: Json<CreateEmissionFactorRequest>,
) -> Result<Json<CreateEmissionFactorResponse>, ApiError> {
    let emission_factor: CreateEmissionFactorResponse = sqlx::query_as!(
        CreateEmissionFactorResponse,
        r"
        insert into emission_factor
        (factor, unit, carrier, source, source_url)
        select $1, $2, energy_carrier.id, $4, $5
        from energy_carrier
        where energy_carrier.name = $3
        returning id, factor, unit, $3 as carrier, source, $5 as source_url
        ",
        request.factor,
        request.unit,
        request.carrier,
        request.source,
        request.source_url.as_deref(),
    )
    .fetch_one(&app_state.db)
    .await?;

    Ok(Json(emission_factor))
}

pub async fn get_emission_factor(
    State(app_state): State<AppState>,
    Query(filter): Query<EmissionFactorFilter>,
) -> Result<Json<Vec<EmissionFactor>>, ApiError> {
    let factors = sqlx::query_as!(
        EmissionFactor,
        r"
        select
            emission_factor.id as id,
            factor,
            unit,
            energy_carrier.name as carrier,
            source,
            source_url,
            emission_factor.updated_at as updated_at,
            emission_factor.created_at as created_at
        from emission_factor
            join energy_carrier on emission_factor.carrier = energy_carrier.id
        where
            ($1::text is null or emission_factor.source = $1) and
            ($2::text is null or energy_carrier.name = $2)
        ",
        filter.source.as_deref(),
        filter.carrier.as_deref(),
    )
    .fetch_all(&app_state.db)
    .await?;

    Ok(Json(factors))
}
