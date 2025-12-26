use crate::db;
use crate::pass_hashing::hash_password;
use rand::RngCore;
use serde::Serialize;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct LoginForm {
    pub username: String,
    pub password: String,
}

pub struct RegisterForm {
    pub username: String,
    pub password: String,
    pub is_admin: bool,
}

pub struct GetUsersForm {
    pub id: Option<u32>,
    pub username: Option<String>,
    pub limit: Option<u32>,
}

#[derive(Serialize)]
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

#[derive(Debug)]
pub enum GetUsersError {
    UnexpectedError,
}

pub async fn get_users(
    pool: &sqlx::Pool<sqlx::MySql>,
    form: GetUsersForm,
) -> Result<Vec<UserReturn>, GetUsersError> {
    let users_req = db::users::get_users(
        pool,
        db::users::GetUsersForm {
            id: form.id,
            username: form.username,
            password: None,
            ..Default::default()
        },
    )
    .await;

    match users_req {
        Ok(res) => Ok(res.into_iter().map(UserReturn::from).collect()),
        Err(db::users::GetUsersError::UnexpectedError) => Err(GetUsersError::UnexpectedError),
    }
}

#[derive(Debug)]
pub enum RegisterError {
    UnexpectedError,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub user: UserReturn,
    pub token: String,
    pub expires_at: i64,
}

fn now_unix() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

fn generate_token() -> String {
    let mut bytes = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut bytes);
    let mut token = String::with_capacity(64);
    for byte in bytes {
        token.push_str(&format!("{:02x}", byte));
    }
    token
}

pub async fn register(
    pool: &sqlx::Pool<sqlx::MySql>,
    form: RegisterForm,
) -> Result<AuthResponse, RegisterError> {
    let username = form.username;
    let hashed = hash_password(form.password);
    db::users::create_user(
        pool,
        db::users::CreateUserForm {
            username: username.clone(),
            password: hashed,
            is_admin: form.is_admin,
        },
    )
    .await
    .map_err(|_| RegisterError::UnexpectedError)?;

    let user = db::users::get_user(
        pool,
        db::users::GetUsersForm {
            username: Some(username),
            ..Default::default()
        },
    )
    .await
    .map_err(|_| RegisterError::UnexpectedError)?;

    let token = generate_token();
    let expires_at = now_unix() + 60 * 60 * 24 * 7;
    db::sessions::create_session(
        pool,
        db::sessions::CreateSessionForm {
            user_id: user.id,
            token: token.clone(),
            expires_at,
        },
    )
    .await
    .map_err(|_| RegisterError::UnexpectedError)?;

    Ok(AuthResponse {
        user: UserReturn::from(user),
        token,
        expires_at,
    })
}

#[derive(Debug)]
pub enum LoginError {
    NotFoundError,
    InvalidPassword,
    UnexpectedError,
}

pub async fn login(
    pool: &sqlx::Pool<sqlx::MySql>,
    form: LoginForm,
) -> Result<AuthResponse, LoginError> {
    let user = db::users::get_user(
        pool,
        db::users::GetUsersForm {
            username: Some(form.username),
            ..Default::default()
        },
    )
    .await;
    let user = match user {
        Ok(val) => val,
        Err(db::users::GetUserError::NoResults) => return Err(LoginError::NotFoundError),
        Err(db::users::GetUserError::NoInfoToGetFromProvided) => {
            return Err(LoginError::UnexpectedError)
        }
        Err(db::users::GetUserError::UnexpectedError) => return Err(LoginError::UnexpectedError),
    };

    let hashed = hash_password(form.password);
    if hashed != user.password {
        return Err(LoginError::InvalidPassword);
    }

    let token = generate_token();
    let expires_at = now_unix() + 60 * 60 * 24 * 7;
    db::sessions::create_session(
        pool,
        db::sessions::CreateSessionForm {
            user_id: user.id,
            token: token.clone(),
            expires_at,
        },
    )
    .await
    .map_err(|_| LoginError::UnexpectedError)?;

    Ok(AuthResponse {
        user: UserReturn::from(user),
        token,
        expires_at,
    })
}

#[derive(Debug)]
pub enum AdminAuthError {
    NotAdmin,
    UnexpectedError,
}

pub async fn authenticate_admin_by_token(
    pool: &sqlx::Pool<sqlx::MySql>,
    token: String,
) -> Result<UserReturn, AdminAuthError> {
    let session = db::sessions::get_session_by_token(pool, token)
        .await
        .map_err(|_| AdminAuthError::UnexpectedError)?;

    if session.expires_at <= now_unix() {
        let _ = db::sessions::delete_session_by_token(pool, session.token).await;
        return Err(AdminAuthError::UnexpectedError);
    }

    let user = db::users::get_user(
        pool,
        db::users::GetUsersForm {
            id: Some(session.user_id),
            ..Default::default()
        },
    )
    .await
    .map_err(|_| AdminAuthError::UnexpectedError)?;

    if !user.is_admin {
        return Err(AdminAuthError::NotAdmin);
    }

    Ok(user)
}

#[derive(Debug)]
pub enum UserAuthError {
    UnexpectedError,
}

pub async fn authenticate_user_by_token(
    pool: &sqlx::Pool<sqlx::MySql>,
    token: String,
) -> Result<UserReturn, UserAuthError> {
    let session = db::sessions::get_session_by_token(pool, token)
        .await
        .map_err(|_| UserAuthError::UnexpectedError)?;

    if session.expires_at <= now_unix() {
        let _ = db::sessions::delete_session_by_token(pool, session.token).await;
        return Err(UserAuthError::UnexpectedError);
    }

    let user = db::users::get_user(
        pool,
        db::users::GetUsersForm {
            id: Some(session.user_id),
            ..Default::default()
        },
    )
    .await
    .map_err(|_| UserAuthError::UnexpectedError)?;

    Ok(UserReturn::from(user))
}
