select
    sum(series_value) as value
from ts
    join meta m on ts.meta_id = m.id
where
    m.consumption = true and
    m.identifier = 'total_load' and
    ts.series_timestamp between $1 and $2
