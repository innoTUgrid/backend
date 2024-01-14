-- grid consumption per energy carrier
select
    local_consumption.bucket,
    local_consumption.bucket_consumption as bucket_consumption,
    local_consumption.unit as consumption_unit,
    carrier_proportion.carrier_proportion,
    carrier_proportion.carrier_name
from
    (
        select
            time_bucket($1::interval, ts.series_timestamp) as bucket,
            sum(ts.series_value) / total.total_sum as carrier_proportion,
            energy_carrier.name as carrier_name
        from ts
                join meta on ts.meta_id = meta.id
                join energy_carrier on meta.carrier = energy_carrier.id
                join emission_factor on energy_carrier.id = emission_factor.carrier
                join
            (
                select
                    time_bucket($1::interval, ts.series_timestamp) as inner_bucket,
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
            ) as total on total.inner_bucket = time_bucket($1::interval, ts.series_timestamp)
        where
            meta.consumption = true and
            energy_carrier.name != 'electricity' and
            ts.series_timestamp between $2::timestamptz and $3::timestamptz
        group by
            bucket,
            total.total_sum,
            carrier_name
        order by
            bucket
    ) as carrier_proportion left join
    (
        select 
            time_bucket($1::interval, ts.series_timestamp) as bucket,
            avg(series_value) as bucket_consumption,
            meta.unit
        from ts
            join meta on ts.meta_id = meta.id
        where
            meta.consumption = true and
            meta.identifier = 'grid_reference_smard' and
            ts.series_timestamp between $2::timestamptz and $3::timestamptz
        group by
            bucket,
            meta.unit
        order by
            bucket
    ) as local_consumption on local_consumption.bucket = carrier_proportion.bucket
order by
    carrier_proportion.bucket;

