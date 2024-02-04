use crate::error::ApiError;
use crate::models::Result;
use axum::extract::State;
use serde_json::Value;
use sqlx::{Pool, Postgres};

use axum::Json;

pub async fn put_config(
    State(pool): State<Pool<Postgres>>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, ApiError> {
    sqlx::query!(
        r#"
        insert into config (id, config) values (1, $1)
        on conflict (id) do update
            set config = $1
        returning config
        "#,
        payload
    )
    .fetch_one(&pool)
    .await?;

    Ok(Json(payload))
}

pub async fn get_config(State(pool): State<Pool<Postgres>>) -> Result<Json<Value>, ApiError> {
    let row = sqlx::query!(
        r#"
        select config from config
        "#,
    )
    .fetch_one(&pool)
    .await?;

    Ok(Json(row.config))
}
