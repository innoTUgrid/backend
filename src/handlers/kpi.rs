use crate::error::ApiError;
use crate::models::EmissionFactorSource;
use crate::models::KpiResult;
use crate::models::TimestampFilter;
use crate::models::{Consumption, ConsumptionByCarrier, EmissionsByCarrier, Resampling, Result};

use crate::cache::Cache;
use axum::extract::{Query, State};
use axum::Json;
use sqlx::{Pool, Postgres};
use std::string::String;

/*
total_load / (locally produced energy)
*/
pub async fn get_consumption_production_ratio(
    timestamp_filter: &TimestampFilter,
    pool: &Pool<Postgres>,
) -> Result<f64> {
    let from_timestamp = timestamp_filter.from.unwrap();
    let to_timestamp = timestamp_filter.to.unwrap();

    let consumption_record = sqlx::query_file!(
        "src/sql/total_consumption.sql",
        from_timestamp,
        to_timestamp,
    )
    .fetch_one(pool)
    .await?;

    let production_record =
        sqlx::query_file!("src/sql/total_production.sql", from_timestamp, to_timestamp,)
            .fetch_one(pool)
            .await?;
    let consumption: f64 = consumption_record.value.unwrap_or(1.0);
    let production: f64 = production_record.value.unwrap_or(1.0);
    let mut consumption_production_ratio = consumption;
    if production != 0.0 {
        consumption_production_ratio = consumption / production;
    }
    Ok(consumption_production_ratio)
}

pub async fn get_self_consumption(
    Query(timestamp_filter): Query<TimestampFilter>,
    State(pool): State<Pool<Postgres>>,
) -> Result<Json<KpiResult>> {
    let consumption_production_ratio =
        get_consumption_production_ratio(&timestamp_filter, &pool).await?;
    let self_consumption = f64::min(consumption_production_ratio, 1.0);
    let kpi_result = KpiResult {
        value: self_consumption,
        name: String::from("self_consumption"),
        unit: None,
        from_timestamp: timestamp_filter.from.unwrap(),
        to_timestamp: timestamp_filter.to.unwrap(),
    };
    Ok(Json(kpi_result))
}

pub async fn get_autarky(
    Query(timestamp_filter): Query<TimestampFilter>,
    State(pool): State<Pool<Postgres>>,
) -> Result<Json<KpiResult>> {
    let consumption_production_ratio =
        get_consumption_production_ratio(&timestamp_filter, &pool).await?;
    let autarky = f64::min(1.0 / consumption_production_ratio, 1.0);
    let kpi_result = KpiResult {
        value: autarky,
        name: String::from("autarky"),
        unit: None,
        from_timestamp: timestamp_filter.from.unwrap(),
        to_timestamp: timestamp_filter.to.unwrap(),
    };
    Ok(Json(kpi_result))
}

/*
return consumption for each carrier as timeseries in kwh
*/
pub async fn get_consumption(
    State(pool): State<Pool<Postgres>>,
    Query(timestamp_filter): Query<TimestampFilter>,
    Query(resampling): Query<Resampling>,
) -> Result<Json<Vec<ConsumptionByCarrier>>> {
    let pg_resampling_interval = resampling.map_interval()?;
    let from_timestamp = timestamp_filter.from.unwrap();
    let to_timestamp = timestamp_filter.to.unwrap();
    let grid_consumption_records: Vec<Consumption> = sqlx::query_file_as!(
        Consumption,
        "src/sql/grid_consumption.sql",
        pg_resampling_interval,
        from_timestamp,
        to_timestamp,
    )
    .fetch_all(&pool)
    .await?;

    let local_production_records: Vec<Consumption> = sqlx::query_file_as!(
        Consumption,
        "src/sql/local_production.sql",
        from_timestamp,
        to_timestamp,
        pg_resampling_interval,
    )
    .fetch_all(&pool)
    .await?;

    let mut kpi_results: Vec<ConsumptionByCarrier> = vec![];
    for consumption in grid_consumption_records {
        let kpi_value = consumption.carrier_proportion.unwrap_or(1.0)
            * consumption.bucket_consumption.unwrap_or(0.0);
        let kpi_result = ConsumptionByCarrier {
            bucket: consumption.bucket.unwrap(),
            value: kpi_value,
            carrier_name: consumption.carrier_name,
            unit: String::from("kwh"),
            local: false,
        };
        kpi_results.push(kpi_result);
    }
    for production in local_production_records {
        let kpi_value = production.carrier_proportion.unwrap_or(1.0)
            * production.bucket_consumption.unwrap_or(0.0);
        let kpi_result = ConsumptionByCarrier {
            bucket: production.bucket.unwrap(),
            value: kpi_value,
            carrier_name: production.carrier_name,
            unit: String::from("kwh"),
            local: true,
        };
        kpi_results.push(kpi_result);
    }
    Ok(Json(kpi_results))
}

/*
return consumption by local consumers as timeseries in kwh
*/
pub async fn get_local_consumption(
    Query(timestamp_filter): Query<TimestampFilter>,
    Query(resampling): Query<Resampling>,
    State(pool): State<Pool<Postgres>>,
) -> Result<Json<Vec<ConsumptionByConsumer>>> {
    if !resampling.validate_interval() {
        return Err(ApiError::InvalidInterval);
    }
    let interval = resampling.map_interval()?;
    let from_timestamp = timestamp_filter.from.unwrap();
    let to_timestamp = timestamp_filter.to.unwrap();

    let consumers_consumption = sqlx::query_file_as!(
        ConsumptionByConsumer,
        "src/sql/local_consumption.sql",
        from_timestamp,
        to_timestamp,
        interval
    )
    .fetch_all(&pool)
    .await?;
    Ok(Json(consumers_consumption))
}

pub async fn get_total_consumption(
    Query(timestamp_filter): Query<TimestampFilter>,
    State(pool): State<Pool<Postgres>>,
) -> Result<Json<KpiResult>, ApiError> {
    let from_timestamp = timestamp_filter.from.unwrap();
    let to_timestamp = timestamp_filter.to.unwrap();

    let consumption_record = sqlx::query_file!(
        "src/sql/total_consumption.sql",
        from_timestamp,
        to_timestamp,
    )
    .fetch_one(&pool)
    .await?;

    let consumption: f64 = consumption_record.value.unwrap_or(0.0);
    let kpi_result = KpiResult {
        value: consumption,
        name: String::from("total_consumption"),
        unit: Some(String::from("kwh")),
        from_timestamp,
        to_timestamp,
    };
    Ok(Json(kpi_result))
}

/*
*/
pub async fn get_total_production(
    Query(timestamp_filter): Query<TimestampFilter>,
    State(pool): State<Pool<Postgres>>,
) -> Result<Json<KpiResult>, ApiError> {
    let from_timestamp = timestamp_filter.from.unwrap();
    let to_timestamp = timestamp_filter.to.unwrap();

    let production_record =
        sqlx::query_file!("src/sql/total_production.sql", from_timestamp, to_timestamp,)
            .fetch_one(&pool)
            .await?;

    let production: f64 = production_record.value.unwrap_or(0.0);
    let kpi_result = KpiResult {
        value: production,
        name: String::from("total_production"),
        unit: Some(String::from("kwh")),
        from_timestamp,
        to_timestamp,
    };
    Ok(Json(kpi_result))
}

pub async fn get_co2_savings(
    Query(timestamp_filter): Query<TimestampFilter>,
    Query(resampling): Query<Resampling>,
    Query(ef_source): Query<EmissionFactorSource>,
    State(pool): State<Pool<Postgres>>,
) -> Result<Json<KpiResult>> {
    let ef_source = ef_source.get_source_or_default(&pool).await?;
    let pg_resampling_interval = resampling.map_interval()?;
    let from_timestamp = timestamp_filter.from.unwrap();
    let to_timestamp = timestamp_filter.to.unwrap();

    let key = format!(
        "co2_savings_{}_{}_{:?}_{}",
        from_timestamp, to_timestamp, pg_resampling_interval, ef_source
    );
    let mut cache = Cache::new().await.unwrap();
    let cached_result = cache.get(&key).await;
    match cached_result {
        Ok(result) => {
            let deserialized = serde_json::from_str(&result).unwrap();
            Ok(Json(deserialized))
        }
        Err(_) => {
            let query_results = sqlx::query_file!(
                "src/sql/co2_savings.sql",
                from_timestamp,
                to_timestamp,
                pg_resampling_interval,
                ef_source
            )
            .fetch_one(&pool)
            .await?;
            let kpi = KpiResult {
                value: query_results.co2_savings.unwrap_or_default(),
                name: String::from("co2_savings"),
                unit: Some(String::from("kgco2eq")),
                from_timestamp: timestamp_filter.from.unwrap(),
                to_timestamp: timestamp_filter.to.unwrap(),
            };
            let serialized = serde_json::to_string(&kpi).unwrap();
            cache.set(&key, &serialized, 5 * 60).await.unwrap();
            Ok(Json(kpi))
        }
    }
}

pub async fn get_cost_savings(
    Query(timestamp_filter): Query<TimestampFilter>,
    State(pool): State<Pool<Postgres>>,
) -> Result<Json<KpiResult>> {
    let from_timestamp = timestamp_filter.from.unwrap();
    let to_timestamp = timestamp_filter.to.unwrap();

    let cost_saving_query_results =
        sqlx::query_file!("src/sql/cost_savings.sql", from_timestamp, to_timestamp,)
            .fetch_one(&pool)
            .await?;

    let kpi = KpiResult {
        value: cost_saving_query_results.cost_savings.unwrap(),
        name: String::from("cost_savings"),
        unit: Some(String::from("EUR")),
        from_timestamp: timestamp_filter.to.unwrap(),
        to_timestamp: timestamp_filter.to.unwrap(),
    };
    Ok(Json(kpi))
}

pub async fn get_scope_one_emissions(
    Query(timestamp_filter): Query<TimestampFilter>,
    Query(resampling): Query<Resampling>,
    Query(ef_source): Query<EmissionFactorSource>,
    State(pool): State<Pool<Postgres>>,
) -> Result<Json<Vec<EmissionsByCarrier>>> {
    let ef_source = ef_source.get_source_or_default(&pool).await?;
    let interval = resampling.map_interval()?;
    let from_timestamp = timestamp_filter.from.unwrap();
    let to_timestamp = timestamp_filter.to.unwrap();
    let mut cache = Cache::new().await.unwrap();
    let key = format!(
        "scope_one_emissions_{}_{}_{:?}_{}",
        timestamp_filter.from.unwrap(),
        timestamp_filter.to.unwrap(),
        resampling,
        ef_source
    );
    let cached_result = cache.get(&key).await;
    match cached_result {
        Ok(result) => {
            let deserialized = serde_json::from_str(&result).unwrap();
            Ok(Json(deserialized))
        }
        Err(_) => {
            if !resampling.validate_interval() {
                return Err(ApiError::InvalidInterval);
            }
            let production_record = sqlx::query_file_as!(
                EmissionsByCarrier,
                "src/sql/scope_one_emissions.sql",
                from_timestamp,
                to_timestamp,
                interval,
                ef_source
            )
            .fetch_all(&pool)
            .await?;
            let serialized = serde_json::to_string(&production_record).unwrap();
            cache.set(&key, &serialized, 5 * 60).await.unwrap();
            Ok(Json(production_record))
        }
    }
}

pub async fn get_scope_two_emissions(
    Query(timestamp_filter): Query<TimestampFilter>,
    Query(resampling): Query<Resampling>,
    Query(emission_factor_source): Query<EmissionFactorSource>,
    State(pool): State<Pool<Postgres>>,
) -> Result<Json<Vec<EmissionsByCarrier>>> {
    if !resampling.validate_interval() {
        return Err(ApiError::InvalidInterval);
    }
    let ef_source = emission_factor_source.get_source_or_default(&pool).await?;
    let pg_resampling_interval = resampling.map_interval()?;
    let from_timestamp = timestamp_filter.from.unwrap();
    let to_timestamp = timestamp_filter.to.unwrap();

    let mut cache = Cache::new().await.unwrap();
    let key = format!(
        "scope_two_emissions_{}_{}_{:?}_{}",
        timestamp_filter.from.unwrap(),
        timestamp_filter.to.unwrap(),
        resampling,
        ef_source
    );
    let cached_result = cache.get(&key).await;
    match cached_result {
        Ok(result) => {
            let deserialized = serde_json::from_str(&result).unwrap();
            Ok(Json(deserialized))
        }
        Err(_) => {
            let consumption_record = sqlx::query_file_as!(
                EmissionsByCarrier,
                "src/sql/scope_two_emissions.sql",
                from_timestamp,
                to_timestamp,
                pg_resampling_interval,
                ef_source
            )
            .fetch_all(&pool)
            .await?;
            let serialized = serde_json::to_string(&consumption_record).unwrap();
            cache.set(&key, &serialized, 5 * 60).await.unwrap();
            Ok(Json(consumption_record))
        }
    }
}

/*
*/
pub async fn get_total_co2_emissions(
    Query(timestamp_filter): Query<TimestampFilter>,
    State(pool): State<Pool<Postgres>>,
    Query(resampling): Query<Resampling>,
    Query(emission_factor_source): Query<EmissionFactorSource>,
) -> Result<Json<KpiResult>, ApiError> {
    if !resampling.validate_interval() {
        return Err(ApiError::InvalidInterval);
    }
    let ef_source = emission_factor_source.get_source_or_default(&pool).await?;
    let pg_resampling_interval = resampling.map_interval()?;
    let from_timestamp = timestamp_filter.from.unwrap();
    let to_timestamp = timestamp_filter.to.unwrap();

    let scope_two = sqlx::query_file_as!(
        EmissionsByCarrier,
        "src/sql/scope_two_emissions.sql",
        from_timestamp,
        to_timestamp,
        pg_resampling_interval,
        ef_source
    )
    .fetch_all(&pool)
    .await?;

    let scope_one = sqlx::query_file_as!(
        EmissionsByCarrier,
        "src/sql/scope_one_emissions.sql",
        from_timestamp,
        to_timestamp,
        pg_resampling_interval,
        ef_source
    )
    .fetch_all(&pool)
    .await?;
    let sum_scope_one: f64 = scope_one
        .iter()
        .map(|emission| emission.value.unwrap_or(0.0))
        .sum();
    let sum_scope_two: f64 = scope_two
        .iter()
        .map(|emission| emission.value.unwrap_or(0.0))
        .sum();

    let total_emissions = sum_scope_one + sum_scope_two;

    let kpi_result = KpiResult {
        value: total_emissions,
        name: String::from("total_co2_emissions"),
        unit: Some(String::from("kgco2eq")),
        from_timestamp: timestamp_filter.from.unwrap(),
        to_timestamp: timestamp_filter.to.unwrap(),
    };
    Ok(Json(kpi_result))
}

pub async fn get_total_grid_electricity_cost(
    Query(timestamp_filter): Query<TimestampFilter>,
    State(pool): State<Pool<Postgres>>,
    Query(resampling): Query<Resampling>,
) -> Result<Json<KpiResult>, ApiError> {
    if !resampling.validate_interval() {
        return Err(ApiError::InvalidInterval);
    }
    let pg_resampling_interval = resampling.map_interval()?;
    let from_timestamp = timestamp_filter.from.unwrap();
    let to_timestamp = timestamp_filter.to.unwrap();

    let total_cost_kpi = sqlx::query_file!(
        "src/sql/total_grid_electricity_cost.sql",
        from_timestamp,
        to_timestamp,
        pg_resampling_interval
    )
    .fetch_one(&pool)
    .await?;
    let kpi_result = KpiResult {
        value: total_cost_kpi.value.unwrap_or(0.0),
        name: String::from("total_grid_electricity_cost"),
        unit: Some(String::from("EUR")),
        from_timestamp: timestamp_filter.from.unwrap(),
        to_timestamp: timestamp_filter.to.unwrap(),
    };
    Ok(Json(kpi_result))
}
