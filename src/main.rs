use crate::infrastructure::{create_connection_pool, create_router, read_log_level};
use tracing_subscriber::fmt;

mod error;
mod handlers;
mod infrastructure;
mod models;

#[tokio::main]
async fn main() {
    let log_level = read_log_level();
    fmt::Subscriber::builder()
        .with_max_level(log_level)
        .init();

    let _pool = create_connection_pool().await;
    let app = create_router(_pool);

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
