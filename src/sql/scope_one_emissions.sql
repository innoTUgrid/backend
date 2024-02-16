with local_production as (
    select
        ts.series_timestamp as timestamp,
        ts.series_value as value,
        meta.identifier as identifier,
        energy_carrier.name as energy_carrier,
        emission_factor.factor as emission_factor,
        CASE
            WHEN LAG(ts.series_timestamp) OVER (PARTITION BY ts.meta_id ORDER BY ts.series_timestamp) IS NOT NULL
                THEN extract(epoch from (ts.series_timestamp - lag(ts.series_timestamp) over (PARTITION BY ts.meta_id order by ts.series_timestamp))) / 3600
            ELSE extract(epoch from (LEAD(ts.series_timestamp) OVER (PARTITION BY ts.meta_id order by ts.series_timestamp)) - ts.series_timestamp) / 3600
            END AS timestamp_distance
    from ts
             join meta on ts.meta_id = meta.id
             join energy_carrier on meta.carrier = energy_carrier.id
             join emission_factor on energy_carrier.id = emission_factor.carrier
    where
        meta.consumption = false and
        meta.local = true and
        emission_factor.source = $4
      and ts.series_timestamp between $1 and $2
), kwh as (
    select
        timestamp,
        value * timestamp_distance as production,
        identifier,
        energy_carrier,
        emission_factor
    from local_production
), production_with_emissions as (
    select
        time_bucket($3::interval, kwh.timestamp) as bucket,
        kwh.identifier                           as source_of_production,
        kwh.energy_carrier                       as production_carrier,
        sum(greatest(kwh.production, 0.0))       as production,
        kwh.emission_factor                      as emission_factor
    from kwh
    group by
        time_bucket($3::interval, kwh.timestamp),
        kwh.energy_carrier,
        kwh.identifier,
        kwh.emission_factor
)
select
    production_with_emissions.bucket as bucket,
    production_with_emissions.production * production_with_emissions.emission_factor as value,
    production_with_emissions.production_carrier as carrier_name,
    'kgco2eq' as unit
from
    production_with_emissions
order by bucket;
