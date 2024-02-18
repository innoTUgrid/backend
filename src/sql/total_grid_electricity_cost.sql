-- compute the total cost of electricity at a given granularity for a given time period
-- we first take the total electricity consumption from the grid and then multiply this by the market price for that period
with grid_electricity_base as (
    select
        ts.series_timestamp as timestamp,
        ts.series_value as value,
        case
            when lag(ts.series_timestamp) over (partition by ts.meta_id order by ts.series_timestamp) is not null
                then extract(epoch from (ts.series_timestamp - lag(ts.series_timestamp) over (partition by ts.meta_id order by ts.series_timestamp))) / 3600
            else extract(epoch from (lead(ts.series_timestamp) over (partition by ts.meta_id order by ts.series_timestamp)) - ts.series_timestamp) / 3600
            end as timestamp_distance
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