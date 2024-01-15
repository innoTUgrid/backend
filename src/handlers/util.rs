use crate::models::PingResponse;

use axum::extract::State;
use axum::Json;

use sqlx::{Pool, Postgres};

pub async fn ping(State(_pool): State<Pool<Postgres>>) -> Json<PingResponse> {
    Json(PingResponse::default())
}
