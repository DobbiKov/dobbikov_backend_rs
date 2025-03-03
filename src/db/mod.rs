pub mod create_tables;
pub mod lecture_notes;
pub mod sections;
pub mod subsections;
pub mod users;

use dotenvy::dotenv;
use std::{default, env};

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

enum VecWrapper {
    String(String),
    Num(u32),
    Bool(bool),
}

#[derive(Default)]
pub enum OrAnd {
    Or,
    #[default]
    And,
}
