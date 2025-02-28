use sqlx::Executor;

pub struct CreateUserForm {
    pub username: String,
    pub password: String,
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
