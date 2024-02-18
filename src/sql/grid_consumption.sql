-- 
-- calc energy in kWh consumed via SMARD by each carrier and its percentage in the energy mix during each interval
--
with local_consumption_intermediate as (
    select
        ts.series_timestamp as timestamp,
        ts.series_value as value,
        meta.unit as unit,
        CASE
            WHEN LAG(ts.series_timestamp) OVER (PARTITION BY ts.meta_id ORDER BY ts.series_timestamp) IS NOT NULL 
            THEN LEAST(extract(epoch FROM (ts.series_timestamp - lag(ts.series_timestamp) over (PARTITION BY ts.meta_id ORDER BY ts.series_timestamp))) / 3600, 0.25)
            ELSE LEAST(extract(epoch FROM (LEAD(ts.series_timestamp) OVER (PARTITION BY ts.meta_id ORDER BY ts.series_timestamp)) - ts.series_timestamp) / 3600, 0.25)
        END AS timestamp_distance
    from ts
        join meta on ts.meta_id = meta.id
    where
        meta.identifier = 'grid_reference_smard'
        and
        ts.series_timestamp between $2 and $3
), 
-- convert to kWh
kwh as (    
    select
        local_consumption_intermediate.timestamp,
        value * timestamp_distance as production,
        unit
    from local_consumption_intermediate
), 
-- group by interval
local_consumption as (
    select
        time_bucket($1::interval, kwh.timestamp) as bucket,
        sum(kwh.production) as bucket_consumption,
        kwh.unit
    from kwh
    group by
        bucket,
        kwh.unit
),
-- get sum of energy produced by SMARD grid during each interval in mWh
total as (
    select
        time_bucket($1::interval, ts.series_timestamp) as inner_bucket,
        sum(ts.series_value) as total_sum
    from ts
        join meta on ts.meta_id = meta.id
        join energy_carrier on meta.carrier = energy_carrier.id
    where
        meta.consumption = true 
        and
        meta.local = false 
        and
        ts.series_timestamp between $2 and $3
    group by
        inner_bucket
),
-- calc percentage of each energy carrier in SMARD mix
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
        meta.consumption = true
        and
        meta.local = false
        and
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
    local_consumption.bucket, carrier_proportion.carrier_name