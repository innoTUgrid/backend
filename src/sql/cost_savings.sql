-- cost savings
-- cost savings can be calculated as the price of the locally produced electricity

with local_production as (
    select
        ts.series_timestamp as timestamp,
        ts.series_value as value,
        meta.identifier as identifier,
        CASE
            WHEN LAG(ts.series_timestamp) OVER (PARTITION BY ts.meta_id ORDER BY ts.series_timestamp) IS NOT NULL 
            THEN extract(epoch from (ts.series_timestamp - lag(ts.series_timestamp) over (PARTITION BY ts.meta_id order by ts.series_timestamp))) / 3600
            ELSE extract(epoch from (LEAD(ts.series_timestamp) OVER (PARTITION BY ts.meta_id order by ts.series_timestamp)) - ts.series_timestamp) / 3600
        END AS timestamp_distance
    from ts
            join meta on ts.meta_id = meta.id
    where
        meta.consumption = false
        and ts.series_timestamp between $1 and $2
), local_production_kwh as (
    select
        timestamp,
        value * timestamp_distance as production
    from local_production
), electricity_price_bucket as (
    select
        time_bucket('1hour', ts.series_timestamp) as bucket,
        avg(ts.series_value) as average_price,
        meta.identifier as identifier,
        meta.unit as unit
    from ts
        join meta on ts.meta_id = meta.id
    where
        meta.identifier = 'smard_market_price' and
        ts.series_timestamp between $1 and $2
    group by
        bucket,
        meta.identifier,
        meta.unit
), local_production_bucket as (
    select
        time_bucket('1hour', local_production_kwh.timestamp) as bucket,
        sum(greatest(local_production_kwh.production, 0)) as production
    from local_production_kwh
    group by
        bucket
), intermediate as (
select
    local_production_bucket.production * (electricity_price_bucket.average_price / 1000) as value,
    local_production_bucket.bucket
from electricity_price_bucket
    join local_production_bucket on electricity_price_bucket.bucket = local_production_bucket.bucket
)
select
    sum(value) as cost_savings
from intermediate;




