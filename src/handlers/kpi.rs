use crate::error::ApiError;
use crate::models::Co2Savings;
use crate::models::KpiResult;
use crate::models::{
    Consumption, ConsumptionByCarrier, ConsumptionWithEmissions, EmissionsByCarrier,
    ProductionWithEmissions, Resampling, Result,
};

use crate::models::TimestampFilter;

use axum::extract::{Query, State};
use axum::Json;

use rand::Rng;
use sqlx::{Pool, Postgres};
use std::string::String;

use tokio::fs;

/*
*/
pub async fn get_self_consumption(
    Query(timestamp_filter): Query<TimestampFilter>,
    State(pool): State<Pool<Postgres>>,
) -> Result<Json<KpiResult>> {
    let from_timestamp = timestamp_filter.from.unwrap();
    let to_timestamp = timestamp_filter.to.unwrap();
    let consumption_record = sqlx::query!(
        r"
        select
            sum(series_value) as value
        from ts
            join meta m on ts.meta_id = m.id
        where
            m.consumption = true and
            m.identifier = 'total_load' and
            ts.series_timestamp between $1::timestamptz and $2::timestamptz
        ",
        from_timestamp,
        to_timestamp,
    )
    .fetch_one(&pool)
    .await?;

    let production_record = sqlx::query!(
        r"
        select sum(series_value) as value
        from ts
            join meta m on ts.meta_id = m.id
        where
            m.consumption = false and
            ts.series_timestamp between $1::timestamptz and $2::timestamptz
        ",
        from_timestamp,
        to_timestamp,
    )
    .fetch_one(&pool)
    .await?;
    let consumption: f64 = consumption_record.value.unwrap_or(1.0);
    let production: f64 = production_record.value.unwrap_or(1.0);
    let self_consumption = f64::min(consumption / production, 1.0);
    let kpi_result = KpiResult {
        value: self_consumption,
        name: String::from("self_consumption"),
        unit: None,
        from_timestamp,
        to_timestamp,
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
    let offset = resampling.hours_per_interval()?;
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

    let local_consumption_records: Vec<Consumption> = sqlx::query_file_as!(
        Consumption,
        "src/sql/local_consumption.sql",
        pg_resampling_interval,
        from_timestamp,
        to_timestamp
    )
    .fetch_all(&pool)
    .await?;

    let mut kpi_results: Vec<ConsumptionByCarrier> = vec![];
    // TODO: not very pretty, duplicate iteration is a code smell imo
    for consumption in grid_consumption_records {
        let kpi_value = consumption.carrier_proportion.unwrap_or(1.0)
            * consumption.bucket_consumption.unwrap_or(0.0)
            * offset;
        let kpi_result = ConsumptionByCarrier {
            bucket: consumption.bucket.unwrap(),
            value: kpi_value,
            carrier_name: consumption.carrier_name,
            unit: String::from("kwh"),
            local: false,
        };
        kpi_results.push(kpi_result);
    }
    for consumption in local_consumption_records {
        let kpi_value = consumption.carrier_proportion.unwrap_or(1.0)
            * consumption.bucket_consumption.unwrap_or(0.0)
            * offset;
        let kpi_result = ConsumptionByCarrier {
            bucket: consumption.bucket.unwrap(),
            value: kpi_value,
            carrier_name: consumption.carrier_name,
            unit: String::from("kwh"),
            local: true,
        };
        kpi_results.push(kpi_result);
    }
    Ok(Json(kpi_results))
}

pub async fn get_total_consumption(
    Query(timestamp_filter): Query<TimestampFilter>,
    State(pool): State<Pool<Postgres>>,
    Query(resampling): Query<Resampling>,
) -> Result<Json<KpiResult>, ApiError> {
    let from_timestamp = timestamp_filter.from.unwrap();
    let to_timestamp = timestamp_filter.to.unwrap();
    let pg_resampling_interval = resampling.map_interval()?;

    let consumption_record = sqlx::query!(
        r"
            select sum(subquery.mean_value) as value from 
                (select
                    time_bucket($3::interval, ts.series_timestamp) as bucket,
                    avg(ts.series_value) as mean_value 
                from ts
                    join meta m on ts.meta_id = m.id
                where 
                m.identifier = 'total_load' and
                ts.series_timestamp >= $1 and ts.series_timestamp <= $2
                group by bucket
                order by bucket) subquery
        ",
        from_timestamp,
        to_timestamp,
        pg_resampling_interval,
    )
    .fetch_one(&pool)
    .await?;

    let consumption: f64 =
        consumption_record.value.unwrap_or(0.0) * resampling.hours_per_interval()?;
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
    Query(resampling): Query<Resampling>,
) -> Result<Json<KpiResult>, ApiError> {
    let from_timestamp = timestamp_filter.from.unwrap();
    let to_timestamp = timestamp_filter.to.unwrap();
    let pg_resampling_interval = resampling.map_interval()?;

    let production_record = sqlx::query!(
        r"
            select 
                sum(subquery.sum_production) as value 
            from (
                select
                    time_bucket($1::interval, ts.series_timestamp) as bucket,
                    sum(ts.series_value) as sum_production 
                from ts
                    join meta m on ts.meta_id = m.id
                where 
                    m.consumption = false 
                    and
                    ts.series_timestamp between $2 and $3
                group by bucket
                order by bucket
            ) subquery
        ",
        pg_resampling_interval,
        from_timestamp,
        to_timestamp,
    )
    .fetch_one(&pool)
    .await?;

    let production: f64 =
        production_record.value.unwrap_or(0.0) * resampling.hours_per_interval()?;
    let kpi_result = KpiResult {
        value: production,
        name: String::from("total_production"),
        unit: Some(String::from("kwh")),
        from_timestamp,
        to_timestamp,
    };
    Ok(Json(kpi_result))
}

pub async fn get_autarky(
    Query(timestamp_filter): Query<TimestampFilter>,
    State(pool): State<Pool<Postgres>>,
) -> Result<Json<KpiResult>> {
    let from_timestamp = timestamp_filter.from.unwrap();
    let to_timestamp = timestamp_filter.to.unwrap();

    let consumption_record = sqlx::query!(
        r"
        select
            sum(series_value) as value
        from ts
            join meta m on ts.meta_id = m.id
        where
            m.consumption = true and
            m.identifier = 'total_load' and
            ts.series_timestamp between $1::timestamptz and $2::timestamptz
        ",
        from_timestamp,
        to_timestamp,
    )
    .fetch_one(&pool)
    .await?;

    let production_record = sqlx::query!(
        r"
        select
            sum(series_value) as value
        from ts
            join meta m on ts.meta_id = m.id
        where
            m.consumption = false and
            ts.series_timestamp between $1::timestamptz and $2::timestamptz
        ",
        from_timestamp,
        to_timestamp,
    )
    .fetch_one(&pool)
    .await?;

    let consumption: f64 = consumption_record.value.unwrap_or(1.0);
    let production: f64 = production_record.value.unwrap_or(1.0);
    let autarky = f64::min(production / consumption, 1.0);
    let kpi_result = KpiResult {
        value: autarky,
        name: String::from("autarky"),
        unit: None,
        from_timestamp,
        to_timestamp,
    };
    Ok(Json(kpi_result))
}

/*
calculate interval adjusted co2 savings as:
co2_savings = (hypothetical_emissions - local_emissions) * offset
where:
local_emissions = avg(total_local_production.production * total_local_production.production_emission_factor)
hypothetical_emissions = sum(production * carrier_proportion * emission_factor)
offset = hours in interval e.g. for interval '15min' equals 15/60 == 0.25
*/
pub async fn get_co2_savings(
    Query(timestamp_filter): Query<TimestampFilter>,
    Query(resampling): Query<Resampling>,
    State(pool): State<Pool<Postgres>>,
) -> Result<Json<KpiResult>> {
    let pg_resampling_interval = resampling.map_interval()?;
    let from_timestamp = timestamp_filter.from.unwrap();
    let to_timestamp = timestamp_filter.to.unwrap();

    let query_results = sqlx::query_file_as!(
        Co2Savings,
        "src/sql/co2_savings.sql",
        pg_resampling_interval,
        from_timestamp,
        to_timestamp,
    )
    .fetch_all(&pool)
    .await?;

    let mut co2_savings = 0.0;
    let offset = resampling.hours_per_interval()?;
    for row in query_results {
        let kpi_value =
            row.hypothetical_emissions.unwrap_or(0.0) - row.local_emissions.unwrap_or(0.0);
        co2_savings += kpi_value * offset;
    }
    /* TODO: remember to check units in all places like this */
    /* TODO: remember to adjust values depending on aggregation period, right now we have e.g. kilowatt-months */
    /* TODO: make all kpi endpoints return timeseries to keep unified format */
    let kpi = KpiResult {
        value: co2_savings,
        name: String::from("co2_savings"),
        unit: Some(String::from("kgco2eq")),
        from_timestamp: timestamp_filter.from.unwrap(),
        to_timestamp: timestamp_filter.to.unwrap(),
    };
    Ok(Json(kpi))
}

pub async fn get_cost_savings(
    Query(timestamp_filter): Query<TimestampFilter>,
    State(_pool): State<Pool<Postgres>>,
) -> Result<Json<KpiResult>> {
    let random_cost_savings = rand::thread_rng().gen_range(0.0..100.0);
    let kpi = KpiResult {
        value: random_cost_savings,
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
    State(pool): State<Pool<Postgres>>,
) -> Result<Json<Vec<EmissionsByCarrier>>> {
    if !resampling.validate_interval() {
        return Err(ApiError::InvalidInterval);
    }

    let interval = resampling.interval.clone();
    let from_timestamp = timestamp_filter.from.unwrap();
    let to_timestamp = timestamp_filter.to.unwrap();

    let mut query = fs::read_to_string("src/sql/scope_one_emissions.sql")
        .await
        .expect("Failed to read SQL file");    

    query = query.replace("{interval}", &interval);

    let production_record = sqlx::query_as::<_, ProductionWithEmissions>(&query)
        .bind(from_timestamp)
        .bind(to_timestamp)
        .fetch_all(&pool)
        .await?;

    let mut kpi_results: Vec<EmissionsByCarrier> = vec![];
    let offset = resampling.hours_per_interval()?;
    for production in production_record {
        let kpi_result = EmissionsByCarrier {
            bucket: production.bucket.unwrap(),
            carrier_name: production.production_carrier,
            value: production.scope_1_emissions.unwrap_or(0.0) * offset,
            unit: String::from("kgco2eq"),
        };
        kpi_results.push(kpi_result);
    }
    Ok(Json(kpi_results))
}

pub async fn get_scope_two_emissions(
    Query(timestamp_filter): Query<TimestampFilter>,
    Query(resampling): Query<Resampling>,
    State(pool): State<Pool<Postgres>>,
) -> Result<Json<Vec<EmissionsByCarrier>>> {
    let pg_resampling_interval = resampling.map_interval()?;
    //let pg_resampling_interval = resampling.interval;
    let from_timestamp = timestamp_filter.from.unwrap();
    let to_timestamp = timestamp_filter.to.unwrap();

    let consumption_record = sqlx::query_file_as!(
        ConsumptionWithEmissions,
        "src/sql/scope_two_emissions.sql",
        pg_resampling_interval,
        from_timestamp,
        to_timestamp,
    )
    .fetch_all(&pool)
    .await?;

    let mut kpi_results: Vec<EmissionsByCarrier> = vec![];
    let offset = resampling.hours_per_interval()?;
    for consumption in consumption_record {
        let kpi_value = consumption.bucket_consumption.unwrap_or(0.0)
            * consumption.carrier_proportion.unwrap_or(1.0)
            * consumption.emission_factor
            * offset;
        let kpi_result = EmissionsByCarrier {
            bucket: consumption.bucket.unwrap(),
            value: kpi_value,
            carrier_name: consumption.carrier_name,
            unit: String::from("kgco2eq"),
        };
        kpi_results.push(kpi_result);
    }
    Ok(Json(kpi_results))
}

/*
*/
pub async fn get_total_co2_emissions(
    Query(timestamp_filter): Query<TimestampFilter>,
    State(pool): State<Pool<Postgres>>,
    Query(resampling): Query<Resampling>,
) -> Result<Json<KpiResult>, ApiError> {
    let from_timestamp = timestamp_filter.from.unwrap();
    let to_timestamp = timestamp_filter.to.unwrap();
    let pg_resampling_interval = resampling.map_interval()?;

    let result = sqlx::query!(
        r"
        select 
            sum(subquery.total_co2_emissions) as total_co2_emissions
        from (
            select
                time_bucket($1, ts.series_timestamp) as bucket,
                -- total co2 emissions
                (avg(greatest(ts.series_value, 0.0)) * avg(emission_factor.factor)) as total_co2_emissions
            from ts
                join meta on ts.meta_id = meta.id
                join energy_carrier on meta.carrier = energy_carrier.id
                join emission_factor on energy_carrier.id = emission_factor.carrier
            where
                ts.series_timestamp between $2 and $3
            group by
                bucket
            order by bucket
        ) subquery;
        ",
        pg_resampling_interval,
        from_timestamp,
        to_timestamp
    )
    .fetch_one(&pool)
    .await?;

    let co2_emissions: f64 =
        result.total_co2_emissions.unwrap_or(0.0) * resampling.hours_per_interval()?;
    let kpi_result = KpiResult {
        value: co2_emissions,
        name: String::from("total_co2_emissions"),
        unit: Some(String::from("kgco2eq")),
        from_timestamp,
        to_timestamp,
    };
    Ok(Json(kpi_result))
}
