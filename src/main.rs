use loggit::{info, logger, Level};
use pass_hashing::hash_password;

pub mod db;
pub mod examples;
pub mod pass_hashing;
pub mod services;

#[cfg(test)]
mod tests;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let pool: sqlx::Pool<sqlx::MySql>;
    logger::set_colorized(true);
    logger::set_level_formatting(
        Level::INFO,
        "<green>[{level}]<green>: {message}".to_string(),
    );

    match db::establish_connection().await {
        Ok(conn) => pool = conn,
        Err(_) => {
            panic!("an error occured")
        }
    };
    info!("The connection was successfully established, checking tables");
    db::create_tables::create_required_tables(&pool).await;
    info!("The tables were verified and the missing ones were successfully created");
}
