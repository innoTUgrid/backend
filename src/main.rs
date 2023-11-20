use crate::infrastructure::{create_connection_pool, create_router};

mod error;
mod handlers;
mod infrastructure;
mod models;

#[tokio::main]
async fn main() {
    let _pool = create_connection_pool().await;
    let app = create_router(_pool);

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
