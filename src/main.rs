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

    match db::establish_connection().await {
        Ok(conn) => pool = conn,
        Err(_) => {
            panic!("an error occured")
        }
    };
    db::create_tables::create_required_tables(&pool).await;
}
