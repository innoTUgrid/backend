use std::env;

use crate::infrastructure::{create_connection_pool, create_router, read_log_level};
use tracing_subscriber::fmt;

mod error;
mod handlers;
mod import;
mod infrastructure;
mod models;
mod tests;

#[tokio::main]
async fn main() {
    let log_level = read_log_level();
    fmt::Subscriber::builder().with_max_level(log_level).init();

    let args: Vec<String> = env::args().collect();
    if args.len() >= 3 && args[1] == "import" {
        let filename = &args[2];
        let mut reader = csv::Reader::from_path(filename).unwrap();
        let _pool = create_connection_pool().await;
        import::import(&_pool, &mut reader).await.unwrap();
        println!("Imported file {}", filename);
        return;
    }
    let _pool = create_connection_pool().await;
    let app = create_router(_pool);

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
