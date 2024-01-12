-- co2 savings
-- first query computes the total local renewable energy production in kwh per energy carrier
-- second query computes the grid proportions per energy carrier together with an emission factor per carrier
-- the co2 savings are the difference between the hypothetical and the actual local emissions
-- TODO: think about how different aggregation periods are meaningful here
-- TODO: missing energy carrier for combined heating and power?
select
    total_local_production.bucket as bucket,
    avg(total_local_production.production * total_local_production.production_emission_factor) as local_emissions,
    sum(production * carrier_proportion * emission_factor) as hypothetical_emissions,
    total_local_production.production_unit as production_unit,
    total_local_production.emission_factor_unit as local_emission_factor_unit,
    grid_proportions.emission_factor_unit as grid_emission_factor_unit


from (
         -- compute the local energy production
         select
             time_bucket($1, ts.series_timestamp) as bucket,
             greatest(avg(ts.series_value), 0) as production,
             energy_carrier.name as production_carrier,
             meta.unit as production_unit,
             emission_factor.factor as production_emission_factor,
             emission_factor.unit as emission_factor_unit
         from ts
                  join meta on ts.meta_id = meta.id
                  join energy_carrier on meta.carrier = energy_carrier.id
                  join emission_factor on emission_factor.carrier = energy_carrier.id
         where
             meta.consumption = false and
             ts.series_timestamp between $2 and $3
         group by
             bucket,
             production_carrier,
             production_unit,
             production_emission_factor,
             emission_factor_unit
         order by bucket
     )
         as total_local_production inner join
     (
         -- compute the grid proportions per energy carrier
         select
             time_bucket($1, ts.series_timestamp) as bucket,
             sum(ts.series_value) / total.total_sum as carrier_proportion,
             energy_carrier.name as carrier_name,
             emission_factor.factor as emission_factor,
             emission_factor.unit as emission_factor_unit
         from ts
                  join meta on ts.meta_id = meta.id
                  join energy_carrier on meta.carrier = energy_carrier.id
                  join emission_factor on energy_carrier.id = emission_factor.carrier
                  join
              (
                  select
                      time_bucket($1, ts.series_timestamp) as inner_bucket,
                      sum(series_value) as total_sum
                  from ts
                           join meta on ts.meta_id = meta.id
                           join energy_carrier on meta.carrier = energy_carrier.id
                  where
                      meta.consumption = true and
                      energy_carrier.name != 'electricity' and
                      ts.series_timestamp between $2 and $3
                  group by
                      inner_bucket
                  order by
                      inner_bucket
              ) as total on total.inner_bucket = time_bucket($1, ts.series_timestamp)
         where
             meta.consumption = true and
             energy_carrier.name != 'electricity' and
             ts.series_timestamp between $2 and $3
         group by
             bucket,
             total.total_sum,
             carrier_name,
             emission_factor,
             emission_factor_unit
         order by
             bucket
     )
         as grid_proportions on total_local_production.bucket = grid_proportions.bucket
group by
    total_local_production.bucket,
    production_unit,
    local_emission_factor_unit,
    grid_emission_factor_unit
order by total_local_production.bucket
