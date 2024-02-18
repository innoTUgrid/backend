-- get energy in kWh consumed by each local consumer during interval
WITH local_consumption AS (
    SELECT
        ts.series_timestamp AS timestamp,
        meta.identifier AS consumer_name,
        ts.series_value AS consumption,
        meta.unit AS unit,
        energy_carrier.name AS energy_carrier,
        CASE
            WHEN LAG(ts.series_timestamp) OVER (PARTITION BY ts.meta_id ORDER BY ts.series_timestamp) IS NOT NULL 
            THEN LEAST(extract(epoch FROM (ts.series_timestamp - lag(ts.series_timestamp) over (PARTITION BY ts.meta_id ORDER BY ts.series_timestamp))) / 3600, 0.25)
            ELSE LEAST(extract(epoch FROM (LEAD(ts.series_timestamp) OVER (PARTITION BY ts.meta_id ORDER BY ts.series_timestamp)) - ts.series_timestamp) / 3600, 0.25)
        END AS timestamp_distance
    FROM ts
        JOIN meta ON ts.meta_id = meta.id
        JOIN energy_carrier ON meta.carrier = energy_carrier.id
    WHERE
        meta.consumption = true 
        AND
        meta.local = true
        AND
        -- damn is this ugly
        meta.identifier NOT IN ('total_load','grid_reference_smard')
        AND 
        ts.series_timestamp BETWEEN $1 AND $2
), 
kwh AS (
    SELECT
        timestamp,
        consumer_name,
        CASE
            WHEN unit = 'w' 
            THEN consumption / 1000 * timestamp_distance
            ELSE consumption * timestamp_distance 
        END AS consumption_in_kwh,
        'kWh' as consumption_unit,
        energy_carrier
    FROM local_consumption
)
SELECT
    time_bucket($3::interval, kwh.timestamp) AS bucket,
    kwh.consumer_name AS consumer_name,
    sum(greatest(kwh.consumption_in_kwh, 0.0)) AS value,
    kwh.consumption_unit AS unit,
    kwh.energy_carrier AS carrier_name
FROM kwh
GROUP BY
    bucket,
    consumer_name,
    energy_carrier,
    consumption_unit
ORDER BY bucket