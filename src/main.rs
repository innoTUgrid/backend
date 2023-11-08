use axum::{routing::get, Router};
use dotenv::dotenv;
use sqlx::Pool;
use sqlx::Postgres;

async fn create_connection_pool() -> Pool<Postgres> {
    dotenv().ok();
    let database_url =
        std::env::var("DATABASE_URL").expect("Couldn't find database url in .env file");
    Pool::<Postgres>::connect(&database_url)
        .await
        .expect("Failed to create database connection pool")
}

#[tokio::main]
async fn main() {
    // build our application with a single route
    let app = Router::new().route("/", get(|| async { "Hello, World!" }));
    let _pool = create_connection_pool().await;

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();

    println!("listening on 0.0.0.0:3000");
}

#[cfg(test)]
mod tests {
    use crate::create_connection_pool;
    #[tokio::test]
    async fn test_create_pool_connection() {
        let pool = create_connection_pool().await;
        let row: (i32,) = sqlx::query_as("SELECT 1")
            .fetch_one(&pool)
            .await
            .expect("Failed to fetch from database");
        assert_eq!(row.0, 1, "Could not connect to database")
    }
}
