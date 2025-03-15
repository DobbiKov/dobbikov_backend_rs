use crate::db;
pub struct LoginForm {
    username: String,
    password: String,
}

pub struct RegisterForm {
    username: String,
    password: String,
}

pub struct GetUsersForm {
    id: Option<u32>,
    username: Option<u32>,
}

pub struct UserReturn {
    pub id: u32,
    pub username: String,
    pub is_admin: bool,
}

impl From<crate::db::users::UserFromDb> for UserReturn {
    fn from(value: crate::db::users::UserFromDb) -> Self {
        Self {
            id: value.id,
            username: value.username,
            is_admin: value.is_admin,
        }
    }
}
pub enum GetUsersError {
    UnexpectedError,
}

pub async fn get_users(
    pool: &sqlx::Pool<sqlx::MySql>,
    form: GetUsersForm,
) -> Result<Vec<UserReturn>, GetUsersError> {
    let users_req = db::users::get_users(pool, Default::default()).await;

    match users_req {
        Ok(res) => Ok(res.into_iter().map(UserReturn::from).collect()),
        Err(e) => match e {
            db::users::GetUsersError::UnexpectedError => Err(GetUsersError::UnexpectedError),
        },
    }
}
