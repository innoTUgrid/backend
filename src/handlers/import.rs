use crate::error::ApiError;
use crate::import::import;

use crate::models::Result;

use axum::extract::Multipart;
use axum::extract::Query;
use axum::extract::State;
use axum::Json;

use sqlx::{Pool, Postgres};
use std::string::String;
/*
upload a file from a form and bulk insert it into the database
docs: https://docs.rs/axum/latest/axum/extract/multipart/struct.Field.html
test: curl -F upload=@initdb/inno2grid_backend_test.csv 127.0.0.1:3000/v1/ts/upload
*/
pub async fn upload_timeseries(
    State(pool): State<Pool<Postgres>>,
    Query(import_config_string): Query<String>,
    mut multipart: Multipart,
) -> Result<Json<String>, ApiError> {
    let import_config = serde_json::from_str(&import_config_string).unwrap();

    while let Some(field) = multipart.next_field().await.unwrap() {
        // whole file is read into memory, which is bad but ok for now
        let text = field.text().await.unwrap();
        let mut reader = csv::ReaderBuilder::new().from_reader(text.as_bytes());

        import(&pool, &mut reader, &import_config).await?;
    }
    Ok(Json("File uploaded successfully".to_string()))
}
