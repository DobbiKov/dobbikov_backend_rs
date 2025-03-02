use pass_hashing::hash_password;

pub mod db;
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

    let users = db::users::get_users(
        &pool,
        db::users::GetUsersForm {
            id: None,
            username: None,
            password: None,
        },
    )
    .await
    .unwrap_or(Vec::new());
    println!("{:?}", users);

    let user = db::users::get_user(
        &pool,
        db::users::GetUsersForm {
            id: None,
            username: Some("hey".to_string()),
            password: None,
        },
    )
    .await;
    if let Ok(res) = user {
        println!("{:?}", res)
    }
}
