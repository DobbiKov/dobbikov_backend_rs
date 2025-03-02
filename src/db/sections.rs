pub struct CreateSectionForm {
    pub title: String,
}

pub struct SectionFromDb {
    pub id: u32,
    pub title: String,
    pub position: i32,
}

pub async fn get_max_position(pool: &sqlx::Pool<sqlx::MySql>) -> Option<u32> {
    let pre_query_str = format!("SELECT MAX(position) FROM sections");
    let query_str = pre_query_str.as_str();
    let query = sqlx::query_scalar(query_str);

    let max: Result<Option<u32>, sqlx::Error> = query.fetch_one(pool).await;
    max.unwrap_or_default()
}

//pub async fn get_users(
//    pool: &sqlx::Pool<sqlx::MySql>,
//    form: GetUsersForm,
//) -> Result<Vec<UserFromDb>, GetUsersError> {
//    let mut conditions: Vec<String> = Vec::new();
//    let mut params: Vec<vecWrapper> = Vec::new();
//
//    if form.id.is_some() {
//        conditions.push("id = ?".to_string());
//        params.push(vecWrapper::Num(form.id.unwrap()));
//    }
//    if form.username.is_some() {
//        conditions.push("username = ?".to_string());
//        params.push(vecWrapper::String(form.username.unwrap()));
//    }
//    if form.password.is_some() {
//        conditions.push("password = ?".to_string());
//        params.push(vecWrapper::String(form.password.unwrap()));
//    }
//
//    let pre_query_str = format!(
//        "SELECT * FROM users {} {}",
//        if !conditions.is_empty() { "WHERE" } else { "" },
//        conditions.join(" AND ")
//    );
//    let query_str = pre_query_str.as_str();
//    println!("{}", query_str);
//    let mut query = sqlx::query_as(query_str);
//
//    for param in params {
//        query = match param {
//            vecWrapper::String(val) => query.bind(val),
//            vecWrapper::Num(val) => query.bind(val),
//            vecWrapper::Bool(val) => query.bind(val),
//        };
//    }
//
//    let users: Vec<UserFromDb> = query.fetch_all(pool).await.unwrap();
//    Ok(users)
//}
//pub async fn create_user(
//    pool: &sqlx::Pool<sqlx::MySql>,
//    user_form: CreateSectionForm,
//) -> Result<(), ()> {
//    let res = sqlx::query!(
//        "INSERT INTO users (username, password) VALUES (?, ?)",
//        user_form.username,
//        user_form.password
//    )
//    .execute(pool)
//    .await;
//    match res {
//        Ok(_) => Ok(()),
//        Err(_) => Err(()),
//    }
//}
