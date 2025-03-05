use crate::db::{self, users::UserFromDb};

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

    let _ = db::users::create_user(
        &pool,
        db::users::CreateUserForm {
            username: "dobb".to_string(),
            password: "pass1".to_string(),
        },
    )
    .await;

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
    assert_eq!(users.len(), 1);

    let _ = db::users::create_user(
        &pool,
        db::users::CreateUserForm {
            username: "ivgap04".to_string(),
            password: "pass2".to_string(),
        },
    )
    .await;

    // testing simple getting all users
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
    assert_eq!(users.len(), 2);

    assert_eq!(
        users,
        vec![
            UserFromDb {
                id: 1,
                username: "dobb".to_string(),
                password: "pass1".to_string(),
                is_admin: false
            },
            UserFromDb {
                id: 2,
                username: "ivgap04".to_string(),
                password: "pass2".to_string(),
                is_admin: false
            },
        ]
    );

    // testing OR
    let users_req = db::users::get_users(
        &pool,
        db::users::GetUsersForm {
            id: None,
            username: Some("dobb".to_string()),
            password: Some("pass2".to_string()),
            or_and: db::OrAnd::Or,
        },
    )
    .await;

    assert!(users_req.is_ok());
    let users = users_req.unwrap_or_default();
    assert_eq!(users.len(), 2);

    assert_eq!(
        users,
        vec![
            UserFromDb {
                id: 1,
                username: "dobb".to_string(),
                password: "pass1".to_string(),
                is_admin: false
            },
            UserFromDb {
                id: 2,
                username: "ivgap04".to_string(),
                password: "pass2".to_string(),
                is_admin: false
            },
        ]
    );

    // testing AND
    let users_req = db::users::get_users(
        &pool,
        db::users::GetUsersForm {
            id: None,
            username: Some("dobb".to_string()),
            password: Some("pass2".to_string()),
            or_and: db::OrAnd::And,
        },
    )
    .await;

    assert!(users_req.is_ok());
    let users = users_req.unwrap_or_default();
    assert_eq!(users.len(), 0);

    // testing AND
    let users_req = db::users::get_users(
        &pool,
        db::users::GetUsersForm {
            id: None,
            username: Some("dobb".to_string()),
            password: Some("pass1".to_string()),
            or_and: db::OrAnd::And,
        },
    )
    .await;

    assert!(users_req.is_ok());
    let users = users_req.unwrap_or_default();
    assert_eq!(users.len(), 1);

    assert_eq!(
        users,
        vec![UserFromDb {
            id: 1,
            username: "dobb".to_string(),
            password: "pass1".to_string(),
            is_admin: false
        },]
    );

    db::create_tables::drop_all_tables(&pool).await;
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

    let _ = db::users::create_user(
        &pool,
        db::users::CreateUserForm {
            username: "ivgap04".to_string(),
            password: "pass2".to_string(),
        },
    )
    .await;

    let user_req = db::users::get_user(
        &pool,
        db::users::GetUsersForm {
            id: None,
            username: Some("ivgap04".to_string()),
            password: None,
            or_and: Default::default(),
        },
    )
    .await;
    assert!(user_req.is_ok());
    let user = user_req.unwrap_or(UserFromDb {
        id: 0,
        username: "".to_string(),
        password: "".to_string(),
        is_admin: false,
    });
    assert_eq!(
        user,
        UserFromDb {
            id: 1,
            username: "ivgap04".to_string(),
            password: "pass2".to_string(),
            is_admin: false
        }
    );
    //if let Ok(res) = user {
    //    println!("{:?}", res)
    //}
    db::create_tables::drop_all_tables(&pool).await;
}
