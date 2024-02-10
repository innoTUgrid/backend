use crate::error::ApiError;

use crate::models::{ImportConfig, MetaInput, NewDatapoint, TimeseriesMeta};

use sqlx::{Pool, Postgres};
use time::OffsetDateTime;

pub async fn import<T: std::io::Read>(
    pool: &Pool<Postgres>,
    readers: Vec<csv::Reader<T>>,
    import_config: &ImportConfig,
) -> Result<(), ApiError> {
    for mut reader in readers {
        let records = reader.records().collect::<Vec<_>>();
        let headers = reader.headers()?.iter().enumerate().collect::<Vec<_>>();

        let time_header = headers
            .iter()
            .find(|(_, header)| header == &import_config.time_column)
            .unwrap()
            .to_owned();

        let mapped_meta_input: Vec<(&MetaInput, Option<usize>)> = import_config
            .timeseries
            .iter()
            .map(|x| {
                (
                    x,
                    headers
                        .iter()
                        .find(|(_, header)| header == &x.identifier)
                        .map(|x| x.0),
                )
            })
            .collect();

        for (meta_input, i) in mapped_meta_input {
            if i.is_none() {
                println!("Column '{}' in not found", meta_input.identifier);
                continue;
            }
            let i = i.unwrap();

            let mut meta_output: Result<TimeseriesMeta, sqlx::Error> = sqlx::query_as!(
                TimeseriesMeta,
                r"
                insert into meta (identifier, unit, carrier, consumption, description)
                select $1, $2, energy_carrier.id, $4, $5
                from energy_carrier
                where energy_carrier.name = $3
                returning id, identifier, unit, $3 as carrier, consumption, description",
                &meta_input.identifier.to_lowercase(),
                &meta_input.unit.to_lowercase(),
                meta_input.carrier.as_deref().unwrap(),
                meta_input.consumption.unwrap(),
                meta_input.description.as_deref().unwrap(),
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
                &meta_input.identifier.to_lowercase(),
                &meta_input.unit.to_lowercase(),
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
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{app_config::AppConfig, infrastructure::create_connection_pool, models::MetaInput};
    use csv::Reader;

    #[tokio::test]
    async fn test_import() {
        let pool = create_connection_pool(&AppConfig::new()).await;
        let import_config = ImportConfig {
            files: Some(vec!["test.csv".to_string()]),
            time_column: "Time".to_string(),
            timeseries: vec![
                MetaInput {
                    identifier: "Production".to_string(),
                    unit: "kW".to_string(),
                    carrier: Some("electricity".to_string()),
                    consumption: Some(false),
                    description: Some("Electricity production".to_string()),
                    local: Some(true),
                },
                MetaInput {
                    identifier: "Consumption".to_string(),
                    unit: "kW".to_string(),
                    carrier: Some("electricity".to_string()),
                    consumption: Some(true),
                    description: Some("Electricity consumption".to_string()),
                    local: Some(true),
                },
            ],
        };
        let mock_csv = r"id,Time,Production,Consumption,2023-01-01 00:00:00+00:00,1.0,2.0";
        let reader = Reader::from_reader(mock_csv.as_bytes());
        import(&pool, vec![reader], &import_config).await.unwrap();
    }
}
