use crate::db;

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
pub async fn get_users_test() {
    let pool: sqlx::Pool<sqlx::MySql>;

    match db::establish_connection_for_testing().await {
        Ok(conn) => pool = conn,
        Err(_) => {
            panic!("an error occured")
        }
    };
    db::create_tables::drop_all_tables(&pool).await;
    db::create_tables::create_required_tables(&pool).await;

    let users_req = db::users::get_users(
        &pool,
        db::users::GetUsersForm {
            id: None,
            username: None,
            password: None,
            or_and: Default::default(),
        },
    )
    .await;
    assert!(users_req.is_ok());
    let users = users_req.unwrap_or_default();
    assert_eq!(users.len(), 0);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
pub async fn get_user_test() {
    let pool: sqlx::Pool<sqlx::MySql>;

    match db::establish_connection_for_testing().await {
        Ok(conn) => pool = conn,
        Err(_) => {
            panic!("an error occured")
        }
    };
    db::create_tables::drop_all_tables(&pool).await;
    db::create_tables::create_required_tables(&pool).await;

    let user_req = db::users::get_user(
        &pool,
        db::users::GetUsersForm {
            id: None,
            username: Some("hey".to_string()),
            password: None,
            or_and: Default::default(),
        },
    )
    .await;
    assert!(user_req.is_err());
    //if let Ok(res) = user {
    //    println!("{:?}", res)
    //}
}
