pub mod create_tables;
use dotenvy::dotenv;
use std::env;

use sqlx::mysql::MySqlPoolOptions;
use sqlx::Connection;

pub async fn establish_connection() -> Result<sqlx::Pool<sqlx::MySql>, sqlx::Error> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    MySqlPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
}
