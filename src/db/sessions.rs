use loggit::trace;

#[derive(sqlx::FromRow, Debug, PartialEq, Eq)]
pub struct SessionFromDb {
    pub id: u32,
    pub user_id: u32,
    pub token: String,
    pub expires_at: i64,
}

pub struct CreateSessionForm {
    pub user_id: u32,
    pub token: String,
    pub expires_at: i64,
}

pub async fn create_session(
    pool: &sqlx::Pool<sqlx::MySql>,
    form: CreateSessionForm,
) -> Result<(), ()> {
    let res = sqlx::query!(
        "INSERT INTO sessions (user_id, token, expires_at) VALUES (?, ?, ?)",
        form.user_id,
        form.token,
        form.expires_at,
    )
    .execute(pool)
    .await;
    res.map_err(|_| ()).map(|_| ())
}

pub async fn get_session_by_token(
    pool: &sqlx::Pool<sqlx::MySql>,
    token: String,
) -> Result<SessionFromDb, ()> {
    let query_str = "SELECT * FROM sessions WHERE token = ? LIMIT 1";
    trace!("{}", query_str);
    let res = sqlx::query_as::<_, SessionFromDb>(query_str)
        .bind(token)
        .fetch_one(pool)
        .await;
    res.map_err(|_| ())
}

pub async fn delete_session_by_token(
    pool: &sqlx::Pool<sqlx::MySql>,
    token: String,
) -> Result<(), ()> {
    let res = sqlx::query!("DELETE FROM sessions WHERE token = ? LIMIT 1", token)
        .execute(pool)
        .await;
    res.map_err(|_| ()).map(|_| ())
}

pub async fn delete_sessions_by_user(
    pool: &sqlx::Pool<sqlx::MySql>,
    user_id: u32,
) -> Result<(), ()> {
    let res = sqlx::query!("DELETE FROM sessions WHERE user_id = ?", user_id)
        .execute(pool)
        .await;
    res.map_err(|_| ()).map(|_| ())
}
