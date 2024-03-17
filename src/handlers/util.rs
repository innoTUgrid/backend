use crate::{infrastructure::AppState, models::PingResponse};

use axum::extract::State;
use axum::Json;

pub async fn ping(State(_app_state): State<AppState>) -> Json<PingResponse> {
    Json(PingResponse::default())
}
