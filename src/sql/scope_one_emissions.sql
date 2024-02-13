-- 
WITH aggregated_data AS (
    SELECT
        time_bucket($3::interval, ts.series_timestamp) AS bucket,
        meta.identifier as source_of_production,
        energy_carrier.name as production_carrier,
        -- aggregations
        avg(greatest(ts.series_value, 0.0)) AS production_mean,
        sum(greatest(ts.series_value, 0.0)) AS production_sum,
        NULLIF(COUNT(ts.series_value) FILTER (WHERE ts.series_value > 0),0) AS nr_of_records,
        EXTRACT(epoch FROM $3::interval) / 3600 AS nr_of_hours,
        -- average result equals actual emission factor for energy_carrier used by source_of_production
        avg(emission_factor.factor) AS emission_factor,
        emission_factor.unit AS emission_factor_unit,
        meta.unit as production_unit
    from ts
        join meta on ts.meta_id = meta.id
        join energy_carrier on meta.carrier = energy_carrier.id
        join emission_factor on energy_carrier.id = emission_factor.carrier
    where
        meta.consumption = false 
        AND
        meta.local = true
        AND
        ts.series_timestamp between $1 and $2
    group by
        bucket,
        source_of_production,
        production_carrier,
        production_unit,
        emission_factor_unit
    order by bucket
)
SELECT 
    bucket,
    source_of_production,
    production_carrier,
    --production_unit,
    -- average kwh produced in time bucket
    production_sum / nr_of_records / nr_of_hours AS production,
    (production_sum / nr_of_records) / nr_of_hours * emission_factor AS scope_one_emissions,
    emission_factor_unit
    --source_of_production,
    --production_carrier,
    --production_mean,
    --production_sum,
    --nr_of_records,
    --nr_of_hours,
FROM aggregated_data;