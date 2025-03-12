use crate::db;

pub async fn get_and_create_user_example() {
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
            or_and: Default::default(),
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
            or_and: Default::default(),
        },
    )
    .await;
    if let Ok(res) = user {
        println!("{:?}", res)
    }
}

pub async fn create_section_example() {
    let pool: sqlx::Pool<sqlx::MySql>;

    match db::establish_connection().await {
        Ok(conn) => pool = conn,
        Err(_) => {
            panic!("an error occured")
        }
    };
    db::create_tables::create_required_tables(&pool).await;

    let _ = db::sections::create_section(
        &pool,
        db::sections::CreateSectionForm {
            title: "new_title_0".to_string(),
        },
    )
    .await;
}

pub async fn get_sections_example() {
    let pool: sqlx::Pool<sqlx::MySql>;

    match db::establish_connection().await {
        Ok(conn) => pool = conn,
        Err(_) => {
            panic!("an error occured")
        }
    };
    db::create_tables::create_required_tables(&pool).await;

    let sections = db::sections::get_sections(
        &pool,
        db::sections::GetSectionsForm {
            id: None,
            title: None,
            position: None,
            ..Default::default()
        },
    )
    .await
    .unwrap_or(Vec::new());
    println!("{:?}", sections);
}
