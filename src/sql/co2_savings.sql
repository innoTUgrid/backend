/*
CO2 savings are the difference between hypothetical and actual local emissions.
Hypothetical emissions are derived from taking the local electricity production and calculating
hypothetical emissions given the energy mix from SMARD, i.e. what if we did not produce any electricity
locally and bought the electricity from the market.
*/
with total_sum as (
    select
        time_bucket($3, ts.series_timestamp) as bucket,
        sum(ts.series_value) as total
    from ts
             join meta on ts.meta_id = meta.id
             join energy_carrier on meta.carrier = energy_carrier.id
    where
        meta.consumption = true and
        meta.local = false and
        ts.series_timestamp between $1 and $2

    group by
        bucket
), carrier_sum as (
    select
        time_bucket($3, ts.series_timestamp) as bucket,
        meta.carrier as carrier,
        sum(ts.series_value) as carrier_total
    from ts
             join meta on ts.meta_id = meta.id
    where
        meta.consumption = true and
        meta.local = false and
        ts.series_timestamp between $1 and $2
    group by
        bucket,
        carrier

), grid_proportion as (
    select
        carrier_sum.bucket as bucket,
        carrier,
        carrier_total / total AS proportion
    from carrier_sum
             join total_sum on carrier_sum.bucket = total_sum.bucket
), local_production_by_carrier as (
    select
        ts.series_timestamp as timestamp,
        ts.series_value as value,
        meta.carrier as carrier,
        CASE
            WHEN LAG(ts.series_timestamp) OVER (PARTITION BY ts.meta_id ORDER BY ts.series_timestamp) IS NOT NULL 
            THEN LEAST(extract(epoch FROM (ts.series_timestamp - lag(ts.series_timestamp) over (PARTITION BY ts.meta_id ORDER BY ts.series_timestamp))) / 3600, 0.25)
            ELSE LEAST(extract(epoch FROM (LEAD(ts.series_timestamp) OVER (PARTITION BY ts.meta_id ORDER BY ts.series_timestamp)) - ts.series_timestamp) / 3600, 0.25)
        END AS timestamp_distance
    from ts
             join meta on ts.meta_id = meta.id
    where
        meta.consumption = false and
        meta.local = true and
        ts.series_timestamp between $1 and $2
), production as (
    select
        time_bucket($3::interval, timestamp) as bucket,
        local_production_by_carrier.carrier as carrier,
        sum(greatest(value * timestamp_distance, 0)) as production
    from
        local_production_by_carrier
    group by
        bucket,
        local_production_by_carrier.carrier
), emissions as (
    select
        factor,
        carrier
    from emission_factor
    where emission_factor.source = $4
), savings as (
    select production.bucket as bucket,
           sum(production.production * grid_proportion.proportion * emissions.factor) as hypothetical_emissions,
           avg(production.production * emissions.factor) as actual_emissions
    from production
             join grid_proportion on production.bucket = grid_proportion.bucket join emissions on grid_proportion.carrier = emissions.carrier
    group by production.bucket
)
select
    sum(hypothetical_emissions - actual_emissions) as co2_savings
from savings;

