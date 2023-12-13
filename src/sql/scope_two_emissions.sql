select
    time_bucket($1::interval, series_timestamp) as bucket,
    sum(ts.series_value) as total_consumption,
    carrier_proportion.consumption_unit,
    carrier_proportion.carrier_name,
    carrier_proportion.carrier_proportion,
    carrier_proportion.emission_factor,
    carrier_proportion.emission_unit
from ts
         join meta on ts.meta_id = meta.id
         join energy_carrier on meta.carrier = energy_carrier.id,
     (
         select
             sum(ts.series_value) / total.total_sum as carrier_proportion,
             energy_carrier.name as carrier_name,
             emission_factor.factor as emission_factor,
             emission_factor.unit as emission_unit,
             meta.unit as consumption_unit
         from ts
                  join meta on ts.meta_id = meta.id
                  join energy_carrier on meta.carrier = energy_carrier.id
                  join emission_factor on energy_carrier.id = emission_factor.carrier,
              (
                  select
                      sum(series_value) as total_sum
                  from ts
                           join meta on ts.meta_id = meta.id
                           join energy_carrier on meta.carrier = energy_carrier.id
                  where
                      meta.consumption = true and
                      energy_carrier.name != 'electricity' and
                      ts.series_timestamp >= $2::timestamptz and
                      ts.series_timestamp <= $3::timestamptz
              ) as total
         where
             meta.consumption = true and
             energy_carrier.name != 'electricity' and
             ts.series_timestamp >= $2::timestamptz and
             ts.series_timestamp <= $3::timestamptz
         group by
             total.total_sum,
             carrier_name,
             emission_factor,
             emission_unit,
             consumption_unit
     ) as carrier_proportion
where
    meta.consumption = true and
    energy_carrier.name = 'electricity' and
    ts.series_timestamp >= $2 and
    ts.series_timestamp <= $3
group by
    bucket,
    carrier_proportion.carrier_proportion,
    carrier_proportion.consumption_unit,
    carrier_proportion.emission_factor,
    carrier_proportion.emission_unit,
    carrier_proportion.carrier_name
order by
    bucket,
    carrier_proportion.carrier_proportion desc;
