-- was local_consumption before, but is local_production, that is locally consumed energy (?)
with local_production as (
    select
        ts.series_timestamp as timestamp,
        ts.series_value as value,
        meta.unit as unit,
        energy_carrier.name as energy_carrier,
        CASE
            WHEN LAG(ts.series_timestamp) OVER (PARTITION BY ts.meta_id ORDER BY ts.series_timestamp) IS NOT NULL 
            THEN extract(epoch from (ts.series_timestamp - lag(ts.series_timestamp) over (PARTITION BY ts.meta_id order by ts.series_timestamp))) / 3600
            ELSE extract(epoch from (LEAD(ts.series_timestamp) OVER (PARTITION BY ts.meta_id order by ts.series_timestamp)) - ts.series_timestamp) / 3600
        END AS timestamp_distance
    from ts
        join meta on ts.meta_id = meta.id
        join energy_carrier on meta.carrier = energy_carrier.id
    where
        meta.consumption = false and
        meta.local = true
        and ts.series_timestamp between $1 and $2
), kwh as (
    select
        timestamp,
        value * timestamp_distance as production,
        unit,
        energy_carrier
    from local_production
)
select 
    time_bucket($3, kwh.timestamp) as bucket,
    sum(greatest(kwh.production, 0.0)) as bucket_consumption,
    kwh.unit as consumption_unit,
    kwh.energy_carrier as carrier_name,
    -- hacky way to reuse consumption struct
    1.0::double precision as carrier_proportion
from kwh
group by
    bucket,
    carrier_name,
    consumption_unit
order by bucket