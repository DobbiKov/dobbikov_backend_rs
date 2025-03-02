use sqlx::{mysql::MySqlRow, prelude::FromRow, Column, Executor, Row};

pub struct CreateUserForm {
    pub username: String,
    pub password: String,
}

#[derive(FromRow, Debug)]
pub struct UserFromDb {
    pub id: i32,
    pub username: String,
    pub password: String,
    pub is_admin: bool,
}

pub struct GetUserForm {
    pub id: Option<u32>,
    pub username: Option<String>,
    pub password: Option<String>,
}

pub enum GetUserError {
    NoInfoToGetFromProvided,
    NoResults,
}

pub async fn create_user(
    pool: &sqlx::Pool<sqlx::MySql>,
    user_form: CreateUserForm,
) -> Result<(), ()> {
    let res = sqlx::query!(
        "INSERT INTO users (username, password) VALUES (?, ?)",
        user_form.username,
        user_form.password
    )
    .execute(pool)
    .await;
    match res {
        Ok(_) => Ok(()),
        Err(_) => Err(()),
    }
}

pub struct GetUsersForm {
    pub id: Option<u32>,
    pub username: Option<String>,
    pub password: Option<String>,
}
pub enum GetUsersError {
    UnexpectedError,
}
enum vecWrapper {
    String(String),
    Num(u32),
    Bool(bool),
}
pub async fn get_users(
    pool: &sqlx::Pool<sqlx::MySql>,
    form: GetUsersForm,
) -> Result<Vec<UserFromDb>, GetUsersError> {
    let mut conditions: Vec<String> = Vec::new();
    let mut params: Vec<vecWrapper> = Vec::new();

    if form.id.is_some() {
        conditions.push("id = ?".to_string());
        params.push(vecWrapper::Num(form.id.unwrap()));
    }
    if form.username.is_some() {
        conditions.push("username = ?".to_string());
        params.push(vecWrapper::String(form.username.unwrap()));
    }
    if form.password.is_some() {
        conditions.push("password = ?".to_string());
        params.push(vecWrapper::String(form.password.unwrap()));
    }

    let pre_query_str = format!(
        "SELECT * FROM users {} {}",
        if !conditions.is_empty() { "WHERE" } else { "" },
        conditions.join(" AND ")
    );
    let query_str = pre_query_str.as_str();
    println!("{}", query_str);
    let mut query = sqlx::query_as(query_str);

    for param in params {
        query = match param {
            vecWrapper::String(val) => query.bind(val),
            vecWrapper::Num(val) => query.bind(val),
            vecWrapper::Bool(val) => query.bind(val),
        };
    }

    let users: Vec<UserFromDb> = query.fetch_all(pool).await.unwrap();
    Ok(users)
}
