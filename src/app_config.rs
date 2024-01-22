use std::env::var;
use tracing::Level;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub database_url: String,
    pub log_level: Level,
    pub port: u16,
    pub run_migrations: bool,
    pub load_initial_data_path: Option<String>,
}

pub fn read_log_level() -> Level {
    let log_level = var("LOG_LEVEL").unwrap_or_else(|_| "INFO".to_string());
    let log_level = match log_level.as_str() {
        "DEBUG" => Level::DEBUG,
        "INFO" => Level::INFO,
        "WARN" => Level::WARN,
        "ERROR" => Level::ERROR,
        "TRACE" => Level::TRACE,
        _ => Level::INFO,
    };
    log_level
}

impl AppConfig {
    pub fn new() -> Self {
        dotenv::dotenv().ok();
        let database_url = var("DATABASE_URL").expect("Need database").to_string();
        let log_level = read_log_level();
        let port = var("BACKEND_PORT")
            .map(|x| x.parse::<u16>().unwrap())
            .unwrap_or(3000);
        let run_migrations = var("RUN_MIGRATIONS")
            .map(|x| x.parse::<bool>().unwrap())
            .unwrap_or(false);
        let load_initial_data_path = var("LOAD_INITIAL_DATA_PATH").ok();
        AppConfig {
            database_url,
            log_level,
            port,
            run_migrations,
            load_initial_data_path,
        }
    }
}
