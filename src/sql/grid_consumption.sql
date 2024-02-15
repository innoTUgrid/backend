-- grid consumption per energy carrier
-- partly copied from scope_one_emissions
with local_consumption_intermediate as (
    select
        ts.series_timestamp as timestamp,
        ts.series_value as value,
        meta.unit as unit,
        CASE
            WHEN LAG(ts.series_timestamp) OVER (PARTITION BY ts.meta_id ORDER BY ts.series_timestamp) IS NOT NULL 
            THEN extract(epoch from (ts.series_timestamp - lag(ts.series_timestamp) over (PARTITION BY ts.meta_id order by ts.series_timestamp))) / 3600
            ELSE extract(epoch from (LEAD(ts.series_timestamp) OVER (PARTITION BY ts.meta_id order by ts.series_timestamp)) - ts.series_timestamp) / 3600
        END AS timestamp_distance
    from ts
        join meta on ts.meta_id = meta.id
    where
        -- TODO: hardcoding the identifier is not nice and hacky
        meta.consumption = true and
        meta.local = true and
        meta.identifier = 'grid_reference_smard' and
        ts.series_timestamp between $2 and $3
), kwh as (
    select
        timestamp,
        value * timestamp_distance as production,
        unit
    from local_consumption_intermediate
), local_consumption as (
    select
        time_bucket($1::interval, timestamp) as bucket,
        sum(kwh.production) as bucket_consumption,
        kwh.unit
    from kwh
    group by
        bucket,
        kwh.unit
),
total as (
    select
        time_bucket($1::interval, ts.series_timestamp) as inner_bucket,
        sum(series_value) as total_sum
    from ts
        join meta on ts.meta_id = meta.id
        join energy_carrier on meta.carrier = energy_carrier.id
    where
        meta.consumption = true and
        meta.local = false and
        energy_carrier.name != 'electricity' and
        ts.series_timestamp between $2 and $3
    group by
        inner_bucket
),
carrier_proportion as (
    select
        time_bucket($1::interval, ts.series_timestamp) as bucket,
        sum(ts.series_value) / total.total_sum as carrier_proportion,
        energy_carrier.name as carrier_name
    from ts
        join meta on ts.meta_id = meta.id
        join energy_carrier on meta.carrier = energy_carrier.id
        join total on total.inner_bucket = time_bucket($1::interval, ts.series_timestamp)
    where
        meta.consumption = true and
        meta.local = false and
        energy_carrier.name != 'electricity' and
        ts.series_timestamp between $2 and $3
    group by
        bucket,
        total.total_sum,
        carrier_name
)
select
    local_consumption.bucket,
    local_consumption.bucket_consumption as bucket_consumption,
    local_consumption.unit as consumption_unit,
    carrier_proportion.carrier_proportion,
    carrier_proportion.carrier_name
from carrier_proportion
    left join local_consumption on local_consumption.bucket = carrier_proportion.bucket
order by
    local_consumption.bucket
