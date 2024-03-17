use std::{
    env::args,
    net::{IpAddr, Ipv4Addr, SocketAddr},
};

use crate::{
    infrastructure::{create_connection_pool, create_router},
    models::ImportConfig,
};
use app_config::AppConfig;

use tracing_subscriber::fmt;

mod app_config;
mod error;
mod handlers;
mod import;
mod infrastructure;
mod loadtest;
mod models;
mod tests;

mod cache;
#[tokio::main]
async fn main() {
    let config = AppConfig::new();
    fmt::Subscriber::builder()
        .with_max_level(config.log_level)
        .init();

    let pool = create_connection_pool(&config).await;
    if config.run_migrations {
        tracing::info!("Running migrations");
        sqlx::migrate!().run(&pool).await.unwrap();
    }

    if config.load_initial_data_path.is_some() {
        let has_ts = sqlx::query!("select id from ts limit 1")
            .fetch_optional(&pool)
            .await
            .unwrap();
        if has_ts.is_some() {
            tracing::info!("Database already contains data. Aborting");
        } else {
            tracing::info!(
                "Loading initial data from {}",
                config.load_initial_data_path.clone().unwrap()
            );
            let meta_reader =
                std::fs::File::open(config.load_initial_data_path.clone().unwrap()).unwrap();
            let import_config: ImportConfig = serde_yaml::from_reader(&meta_reader).unwrap();

            let mut readers = vec![];
            for file in import_config.files.clone().unwrap() {
                let reader = csv::Reader::from_path(file).unwrap();
                readers.push(reader);
            }

            import::import(&pool, readers, &import_config)
                .await
                .unwrap();
        }
    }

    let args: Vec<String> = args().collect();
    if args.len() >= 2 && args[1] == "init" {
        tracing::info!("App initialized");
        return;
    }

    let app = create_router(pool, &config);

    tracing::info!("Listening on port {}", config.port);
    // run it with hyper on localhost:3000
    let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 3000);
    axum::Server::bind(&socket)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
