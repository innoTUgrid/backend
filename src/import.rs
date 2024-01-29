use crate::error::ApiError;

use crate::models::{MetaInput, NewDatapoint, TimeseriesMeta};

use sqlx::{Pool, Postgres};
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
        // look for headers like "Production#electricity_kW"
        let split = header.rsplitn(2, '#').collect::<Vec<_>>();
        if split.len() != 2 {
            continue;
        }

        let name = split[1];
        let carrier_and_unit = split[0];
        let split = carrier_and_unit.rsplitn(2, '_').collect::<Vec<_>>();
        if split.len() != 2 {
            continue;
        }
        let unit = split[0];
        let carrier = split[1];

        sqlx::query!("select name from energy_carrier where name = $1", carrier)
            .fetch_one(pool)
            .await
            .map_err(|_| {
                ApiError::Anyhow(anyhow::Error::msg(format!(
                    "Carrier '{}' not found",
                    carrier
                )))
            })?;

        let meta_input = MetaInput {
            identifier: name.to_lowercase(),
            unit: unit.to_lowercase(),
            carrier: Some(carrier.to_string()),
            consumption: Some(!name.to_lowercase().contains("production")),
            description: Some("description".to_string()),
        };
        let mut meta_output: Result<TimeseriesMeta, sqlx::Error> = sqlx::query_as!(
            TimeseriesMeta,
            r"
                insert into meta (identifier, unit, carrier, consumption, description)
                select $1, $2, energy_carrier.id, $4, $5
                from energy_carrier
                where energy_carrier.name = $3
                returning id, identifier, unit, $3 as carrier, consumption, description",
            &meta_input.identifier,
            &meta_input.unit,
            meta_input.carrier.unwrap(),
            meta_input.consumption.unwrap(),
            meta_input.description.unwrap(),
        )
        .fetch_one(pool)
        .await;

        if meta_output.is_err() {
            meta_output = sqlx::query_as!(
                TimeseriesMeta,
                r"
                    select meta.id, identifier, unit, energy_carrier.name as carrier, consumption, description
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
        sqlx::query!(
            r#"
                insert into ts (series_timestamp, series_value, meta_id)
                (select * from unnest($1::timestamptz[], $2::float[], $3::int[]))
                "#,
            &entries.iter().map(|x| x.timestamp).collect::<Vec<_>>(),
            &entries.iter().map(|x| x.value).collect::<Vec<_>>(),
            &std::iter::repeat(meta_output.id)
                .take(entries.len())
                .collect::<Vec<_>>(),
        )
        .execute(pool)
        .await?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{app_config::AppConfig, infrastructure::create_connection_pool};
    use csv::Reader;

    #[tokio::test]
    async fn test_import() {
        let pool = create_connection_pool(&AppConfig::new()).await;
        let mock_csv = r"
id,Time,Production#electricity_kW,Consumption#biomass_kW
1,2020-01-01 00:00:00+00:00,1.0,2.0";
        let mut reader = Reader::from_reader(mock_csv.as_bytes());
        import(&pool, &mut reader).await.unwrap();
    }
}
