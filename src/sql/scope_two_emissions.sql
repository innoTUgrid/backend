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
), consumption_by_carrier as (
    select
        ts.series_timestamp as timestamp,
        ts.series_value as value,
        meta.carrier as carrier,
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
), consumption as (
    select
        time_bucket($3::interval, timestamp) as bucket,
        consumption_by_carrier.carrier as carrier,
        sum(greatest(value * timestamp_distance, 0)) as consumption
    from
        consumption_by_carrier
    group by
        bucket,
        consumption_by_carrier.carrier
), emissions as (
    select
        factor,
        carrier
    from emission_factor
    where emission_factor.source = $4
)
select
    consumption.bucket as bucket,
    consumption.consumption * grid_proportion.proportion as value,
    energy_carrier.name as carrier_name,
    'kgco2eq' as unit
from grid_proportion
         left join consumption on consumption.bucket = grid_proportion.bucket
         join emissions on grid_proportion.carrier = emissions.carrier
         join energy_carrier on emissions.carrier = energy_carrier.id
order by consumption.bucket;
