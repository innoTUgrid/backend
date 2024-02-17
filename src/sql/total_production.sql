--
-- get total sum of energy in kWh produced
---
with producers_ts as (
    select
        ts.series_timestamp as timestamp,
        ts.series_value as value,
        CASE
            WHEN LAG(ts.series_timestamp) OVER (PARTITION BY ts.meta_id ORDER BY ts.series_timestamp) IS NOT NULL 
            THEN extract(epoch from (ts.series_timestamp - lag(ts.series_timestamp) over (PARTITION BY ts.meta_id order by ts.series_timestamp))) / 3600
            ELSE extract(epoch from (LEAD(ts.series_timestamp) OVER (PARTITION BY ts.meta_id order by ts.series_timestamp)) - ts.series_timestamp) / 3600
        END AS timestamp_distance
    from ts
        join meta on ts.meta_id = meta.id
    where
        meta.consumption = false and
        ts.series_timestamp between $1 and $2
)
select 
    sum(value * timestamp_distance) as value
from producers_ts