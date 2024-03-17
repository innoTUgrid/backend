use crate::error::ApiError;
use crate::infrastructure::AppState;
use crate::models::Result;
use axum::extract::State;
use serde_json::Value;

use axum::Json;

pub async fn put_config(
    State(app_state): State<AppState>,
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
    .fetch_one(&app_state.db)
    .await?;

    Ok(Json(payload))
}

pub async fn get_config(State(app_state): State<AppState>) -> Result<Json<Value>, ApiError> {
    let row = sqlx::query!(
        r#"
        select config from config
        "#,
    )
    .fetch_one(&app_state.db)
    .await?;

    Ok(Json(row.config))
}
