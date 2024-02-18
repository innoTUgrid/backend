-- compute the total cost of electricity at a given granularity for a given time period
-- we first take the total electricity consumption from the grid and then multiply this by the market price for that period
with grid_electricity_base as (
    select
        ts.series_timestamp as timestamp,
        ts.series_value as value,
        CASE
            WHEN LAG(ts.series_timestamp) OVER (PARTITION BY ts.meta_id ORDER BY ts.series_timestamp) IS NOT NULL 
            THEN LEAST(extract(epoch FROM (ts.series_timestamp - lag(ts.series_timestamp) over (PARTITION BY ts.meta_id ORDER BY ts.series_timestamp))) / 3600, 0.25)
            ELSE LEAST(extract(epoch FROM (LEAD(ts.series_timestamp) OVER (PARTITION BY ts.meta_id ORDER BY ts.series_timestamp)) - ts.series_timestamp) / 3600, 0.25)
        END AS timestamp_distance
    from ts
             join meta on ts.meta_id = meta.id
    where
        meta.identifier = 'grid_reference_smard' and
        ts.series_timestamp between $1 and $2
), grid_electricity_kwh as (
    select
        time_bucket($3::interval, timestamp) as bucket,
        sum(greatest(value * timestamp_distance, 0)) as grid_electricity
    from grid_electricity_base
    group by bucket
), electricity_prices as (
    select
        time_bucket($3::interval, ts.series_timestamp) as bucket,
        -- prices are in eur/mwh
        avg(ts.series_value) / 1000 as price
    from ts
        join meta on ts.meta_id = meta.id
    where
        meta.identifier = 'smard_market_price'
    group by bucket
)
select
    sum(grid_electricity_kwh.grid_electricity * electricity_prices.price) as value
from grid_electricity_kwh
     join electricity_prices on grid_electricity_kwh.bucket = electricity_prices.bucket;