use crate::error::ApiError;

use crate::models::NewDatapoint;
use crate::models::{Datapoint, MetaInput, MetaOutput, Result};

use sqlx::{Pool, Postgres, Row};
use time::OffsetDateTime;

pub async fn import<T: std::io::Read>(
    pool: &Pool<Postgres>,
    reader: &mut csv::Reader<T>,
) -> Result<(), ApiError> {
    let records = reader.records().collect::<Vec<_>>();
    let headers = reader.headers()?.iter().enumerate().collect::<Vec<_>>();

    let time_header = headers
        .iter()
        .find(|(_, header)| header == &"Time")
        .unwrap()
        .to_owned();

    for (i, header) in headers {
        // look for headers like "Production_kW"
        let split = header.rsplitn(2, '_').collect::<Vec<_>>();
        if split.len() != 2 {
            continue;
        }

        let name = split[1];
        let unit = split[0];

        let meta_input = MetaInput {
            identifier: name.to_string(),
            unit: unit.to_string(),
            carrier: Some(String::from("oil")),
        };
        let mut meta_output: Result<MetaOutput, sqlx::Error> = sqlx::query_as!(
            MetaOutput,
            r"
                insert into meta (identifier, unit, carrier)
                select $1, $2, energy_carrier.id
                from energy_carrier
                where energy_carrier.name = $3
                returning id, identifier, unit, $3 as carrier",
            &meta_input.identifier,
            &meta_input.unit,
            meta_input.carrier.as_deref(),
        )
        .fetch_one(pool)
        .await;

        if meta_output.is_err() {
            meta_output = sqlx::query_as!(
                MetaOutput,
                r"
                    select meta.id, identifier, unit, energy_carrier.name as carrier
                    from meta
                    inner join energy_carrier on energy_carrier.id = meta.carrier
                    where identifier = $1 and unit = $2
                    ",
                &meta_input.identifier,
                &meta_input.unit,
            )
            .fetch_one(pool)
            .await;
        };

        let meta_output = meta_output?;

        let mut entries = vec![];
        // create timeseries for each row
        for result in records.iter() {
            let record = result.as_ref().unwrap();
            let value = record.get(i).unwrap();
            let time = record.get(time_header.0).unwrap();

            use time::macros::format_description;
            let format = format_description!("[year]-[month]-[day] [hour]:[minute]:[second][offset_hour sign:mandatory]:[offset_minute]");

            entries.push(NewDatapoint {
                timestamp: OffsetDateTime::parse(time, &format).unwrap(),
                value: value.parse::<f64>().unwrap_or(0.0),
                identifier: meta_output.identifier.clone(),
            });
        }

        // wow this is ultra smart
        // https://klotzandrew.com/blog/postgres-passing-65535-parameter-limit
        let _datapoints: Vec<Datapoint> = sqlx::query_as!(
            Datapoint,
            r#"
                insert into ts (series_timestamp, series_value, meta_id)
                (select * from unnest($1::timestamptz[], $2::float[], $3::int[]))
                returning id, series_timestamp as timestamp, series_value as value, created_at, updated_at
                "#,
            &entries.iter().map(|x| x.timestamp).collect::<Vec<_>>(),
            &entries.iter().map(|x| x.value).collect::<Vec<_>>(),
            &std::iter::repeat(meta_output.id)
                .take(entries.len())
                .collect::<Vec<_>>(),
        )
        .fetch_all(pool)
        .await?;
        // we could return the datapoints here, but we don't need them
    }

    Ok(())
}
