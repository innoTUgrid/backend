with local_production as (
    select
        ts.series_timestamp as timestamp,
        ts.series_value as value,
        meta.identifier as identifier,
        energy_carrier.name as energy_carrier,
        emission_factor.factor as emission_factor,
        emission_factor.unit as emission_factor_unit,
        extract(epoch from (ts.series_timestamp - lag(ts.series_timestamp) over (order by ts.series_timestamp))) / 3600 as timestamp_distance
    from ts
        join meta on ts.meta_id = meta.id
        join energy_carrier on meta.carrier = energy_carrier.id
        join emission_factor on energy_carrier.id = emission_factor.carrier
    where
        meta.consumption = false
        and ts.series_timestamp between $1 and $2
), kwh as (
    select
        timestamp,
        value * timestamp_distance as production,
        identifier,
        energy_carrier,
        emission_factor,
        emission_factor_unit
    from local_production
), production_with_emissions as (
     select
         time_bucket($3::interval, kwh.timestamp) as bucket,
         kwh.identifier                           as source_of_production,
         kwh.energy_carrier                       as production_carrier,
         sum(greatest(kwh.production, 0.0))       as production,
         kwh.emission_factor                      as emission_factor,
         kwh.emission_factor_unit                 as emission_factor_unit
     from kwh
     group by
         kwh.energy_carrier,
         kwh.identifier,
         time_bucket($3::interval, kwh.timestamp),
         kwh.emission_factor,
         kwh.emission_factor_unit
 )
select
    production_with_emissions.production * production_with_emissions.emission_factor as scope_one_emissions,
    production_with_emissions.bucket as bucket,
    production_with_emissions.source_of_production as source_of_production,
    production_with_emissions.production_carrier as production_carrier,
    production_with_emissions.production as production,
    production_with_emissions.emission_factor_unit
from
    production_with_emissions
order by bucket