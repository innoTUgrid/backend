-- carrier_proportion_with_emission_factor: calculates the proportion of each energy carrier in the total energy consumption (excluding electricity).   
-- local_consumption: calculates the average consumption for each time bucket.
-- finally LEFT JOIN carrier_proportion_with_emission_factor on local_consumption thus including all entries from local_consumption even for missing entries in carrier_proportion_with_emission_factor

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
), local_consumption as (
    select
        time_bucket($1::interval, timestamp) as bucket,
        avg(value * timestamp_distance) as bucket_consumption,
        local_consumption_intermediate.unit
    from local_consumption_intermediate
    group by
        bucket,
        local_consumption_intermediate.unit
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
carrier_proportion_with_emission_factor as (
    select
        time_bucket($1::interval, ts.series_timestamp) as bucket,
        sum(ts.series_value) / total.total_sum as carrier_proportion,
        energy_carrier.name as carrier_name,
        emission_factor.factor as emission_factor,
        emission_factor.unit as emission_unit
    from ts
        join meta on ts.meta_id = meta.id
        join energy_carrier on meta.carrier = energy_carrier.id
        join emission_factor on energy_carrier.id = emission_factor.carrier
        join total on total.inner_bucket = time_bucket($1::interval, ts.series_timestamp)
    where
        meta.consumption = true and
        meta.local = false and
        energy_carrier.name != 'electricity' and
        ts.series_timestamp between $2 and $3
    group by
        bucket,
        total.total_sum,
        carrier_name,
        emission_factor,
        emission_unit
)
select
    local_consumption.bucket,
    local_consumption.bucket_consumption,
    local_consumption.unit as consumption_unit,
    carrier_proportion_with_emission_factor.carrier_proportion,
    carrier_proportion_with_emission_factor.carrier_name,
    carrier_proportion_with_emission_factor.emission_factor,
    carrier_proportion_with_emission_factor.emission_unit
from carrier_proportion_with_emission_factor
    left join local_consumption on local_consumption.bucket = carrier_proportion_with_emission_factor.bucket
order by
    local_consumption.bucket
