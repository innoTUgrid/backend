select
    local_consumption.bucket,
    local_consumption.bucket_consumption,
    local_consumption.unit as consumption_unit,
    carrier_proportion_with_emission_factor.carrier_proportion,
    carrier_proportion_with_emission_factor.carrier_name,
    carrier_proportion_with_emission_factor.emission_factor,
    carrier_proportion_with_emission_factor.emission_unit
from
    (
         select
             time_bucket($1, ts.series_timestamp) as bucket,
             sum(ts.series_value) / total.total_sum as carrier_proportion,
             energy_carrier.name as carrier_name,
             emission_factor.factor as emission_factor,
             emission_factor.unit as emission_unit
         from ts
                  join meta on ts.meta_id = meta.id
                  join energy_carrier on meta.carrier = energy_carrier.id
                  join emission_factor on energy_carrier.id = emission_factor.carrier
                  join (
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
             emission_unit
     ) as carrier_proportion_with_emission_factor
    left join (
        select time_bucket($1, ts.series_timestamp) as bucket,
               avg(series_value) as bucket_consumption,
               meta.unit
        from ts
                 join meta on ts.meta_id = meta.id
        where
            -- TODO: this might be wrong and is definitely hacky
            meta.consumption = true and
            meta.identifier = 'total_load' and
            ts.series_timestamp between $2 and $3
        group by
            bucket,
            meta.unit
    ) as local_consumption on local_consumption.bucket = carrier_proportion_with_emission_factor.bucket
order by
    local_consumption.bucket
