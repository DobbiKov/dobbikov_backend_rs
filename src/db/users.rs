use crate::db::VecWrapper;
use loggit::trace;
use sqlx::{mysql::MySqlRow, prelude::FromRow, Column, Executor, Row};

use super::OrAnd;

pub struct CreateUserForm {
    pub username: String,
    pub password: String,
}

#[derive(FromRow, Debug, PartialEq, Eq)]
pub struct UserFromDb {
    pub id: u32,
    pub username: String,
    pub password: String,
    pub is_admin: bool,
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

#[derive(Default)]
pub struct GetUsersForm {
    pub id: Option<u32>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub or_and: OrAnd,
}

impl GetUsersForm {
    pub fn is_all_none(&self) -> bool {
        self.id.is_none() && self.username.is_none() && self.password.is_none()
    }
}

pub enum GetUsersError {
    UnexpectedError,
}
pub async fn get_users(
    pool: &sqlx::Pool<sqlx::MySql>,
    form: GetUsersForm,
) -> Result<Vec<UserFromDb>, GetUsersError> {
    let mut conditions: Vec<String> = Vec::new();
    let mut params: Vec<VecWrapper> = Vec::new();

    if form.id.is_some() {
        conditions.push("id = ?".to_string());
        params.push(VecWrapper::Num(form.id.unwrap()));
    }
    if form.username.is_some() {
        conditions.push("username = ?".to_string());
        params.push(VecWrapper::String(form.username.unwrap()));
    }
    if form.password.is_some() {
        conditions.push("password = ?".to_string());
        params.push(VecWrapper::String(form.password.unwrap()));
    }

    let pre_query_str = format!(
        "SELECT * FROM users {} {}",
        if !conditions.is_empty() { "WHERE" } else { "" },
        conditions.join(match form.or_and {
            OrAnd::And => " AND ",
            OrAnd::Or => " OR ",
        })
    );
    let query_str = pre_query_str.as_str();
    trace!("{}", query_str);
    let mut query = sqlx::query_as(query_str);

    for param in params {
        query = match param {
            VecWrapper::String(val) => query.bind(val),
            VecWrapper::Num(val) => query.bind(val),
            VecWrapper::Bool(val) => query.bind(val),
        };
    }

    let users: Vec<UserFromDb> = query.fetch_all(pool).await.unwrap();
    Ok(users)
}

pub enum GetUserError {
    NoInfoToGetFromProvided,
    NoResults,
    UnexpectedError,
}

pub async fn get_user(
    pool: &sqlx::Pool<sqlx::MySql>,
    form: GetUsersForm,
) -> Result<UserFromDb, GetUserError> {
    if form.is_all_none() {
        return Err(GetUserError::NoInfoToGetFromProvided);
    }
    let res = get_users(pool, form).await;
    if res.is_err() {
        return Err(GetUserError::UnexpectedError);
    }
    let mut res = res.unwrap_or(Vec::new());
    if res.is_empty() {
        return Err(GetUserError::NoResults);
    }
    let ret_res = res.swap_remove(0);
    Ok(ret_res)
}
