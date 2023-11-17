-- KPI: self consumption
select sum(value)
from ts join meta m on ts.meta_id = m.id
where m.consumption = True and time <= $1 and time >= $2
group by carrier
