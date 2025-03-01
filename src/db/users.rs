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
pub async fn get_users(
    pool: &sqlx::Pool<sqlx::MySql>,
    form: GetUsersForm,
) -> Result<Vec<UserFromDb>, GetUsersError> {
    let mut after_where: String = "".to_string();
    let mut smth_before: bool = false;
    if form.id.is_some() {
        if smth_before {
            after_where += " AND ";
        }
        after_where += "id = ?";
        smth_before = true;
    }
    if form.username.is_some() {
        if smth_before {
            after_where += " AND ";
        }
        after_where += "username = ?";
        smth_before = true;
    }
    if form.password.is_some() {
        if smth_before {
            after_where += " AND ";
        }
        after_where += "password = ?";
        smth_before = true;
    }

    let pre_query_str = format!(
        "SELECT * FROM users {} {}",
        if smth_before { "WHERE" } else { "" },
        after_where
    );
    let query_str = pre_query_str.as_str();
    println!("{}", query_str);
    let mut query = sqlx::query_as(query_str);

    if form.id.is_some() {
        query = query.bind(form.id.unwrap());
    }
    if form.username.is_some() {
        query = query.bind(form.username.unwrap());
    }
    if form.password.is_some() {
        query = query.bind(form.password.unwrap());
    }

    let users: Vec<UserFromDb> = query.fetch_all(pool).await.unwrap();
    Ok(users)
}
