select
    time_bucket($1, ts.series_timestamp) as bucket,
    greatest(avg(ts.series_value), 0) as bucket_consumption,
    meta.unit as consumption_unit,
    energy_carrier.name as carrier_name,
    -- hacky way to reuse consumption struct
    1.0::double precision as carrier_proportion

from ts
    join meta on ts.meta_id = meta.id
    join energy_carrier on meta.carrier = energy_carrier.id
where
    meta.consumption = false and
    ts.series_timestamp between $2 and $3
group by
    bucket,
    carrier_name,
    consumption_unit
order by bucket
