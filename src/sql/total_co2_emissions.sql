-- total co2 emissions
select
    time_bucket($1, ts.series_timestamp) as bucket,
    meta.identifier as source_of_production,
    energy_carrier.name as production_carrier,
    -- average production per identifier over interval to account for KwH
    avg(greatest(ts.series_value, 0.0)) as production,
    meta.unit as production_unit,
    -- scope 1 emissions
    (avg(greatest(ts.series_value, 0.0)) * avg(emission_factor.factor)) as scope_1_emissions,
    emission_factor.unit as emission_factor_unit
from ts
    join meta on ts.meta_id = meta.id
    join energy_carrier on meta.carrier = energy_carrier.id
    join emission_factor on energy_carrier.id = emission_factor.carrier
where
    meta.consumption = false and
    ts.series_timestamp between $2 and $3
group by
    bucket,
    source_of_production,
    production_carrier,
    production_unit,
    emission_factor_unit
order by bucket