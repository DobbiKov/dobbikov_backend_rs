use crate::db::{OrAnd, VecWrapper};

pub struct CreateSectionForm {
    pub title: String,
}

#[derive(sqlx::FromRow, Debug, PartialEq, Eq)]
pub struct SectionFromDb {
    pub id: u32,
    pub title: String,
    pub position: u32,
}

pub async fn get_max_position(pool: &sqlx::Pool<sqlx::MySql>) -> Option<u32> {
    let pre_query_str = format!("SELECT MAX(position) FROM sections");
    let query_str = pre_query_str.as_str();
    let query = sqlx::query_scalar(query_str);

    let max: Result<Option<u32>, sqlx::Error> = query.fetch_one(pool).await;
    max.unwrap_or(None)
}

pub struct GetSectionsForm {
    pub id: Option<u32>,
    pub title: Option<String>,
    pub position: Option<u32>,
    pub or_and: OrAnd,
}

pub enum GetSectionsError {
    UnexpectedError,
}

pub async fn get_sections(
    pool: &sqlx::Pool<sqlx::MySql>,
    form: GetSectionsForm,
) -> Result<Vec<SectionFromDb>, GetSectionsError> {
    let mut conditions: Vec<String> = Vec::new();
    let mut params: Vec<VecWrapper> = Vec::new();

    if form.id.is_some() {
        conditions.push("id = ?".to_string());
        params.push(VecWrapper::Num(form.id.unwrap()));
    }
    if form.title.is_some() {
        conditions.push("title = ?".to_string());
        params.push(VecWrapper::String(form.title.unwrap()));
    }
    if form.position.is_some() {
        conditions.push("position = ?".to_string());
        params.push(VecWrapper::Num(form.position.unwrap()));
    }

    let pre_query_str = format!(
        "SELECT * FROM sections {} {}",
        if !conditions.is_empty() { "WHERE" } else { "" },
        conditions.join(match form.or_and {
            OrAnd::And => " AND ",
            OrAnd::Or => " OR ",
        })
    );
    let query_str = pre_query_str.as_str();
    println!("{}", query_str);
    let mut query = sqlx::query_as(query_str);

    for param in params {
        query = match param {
            VecWrapper::String(val) => query.bind(val),
            VecWrapper::Num(val) => query.bind(val),
            VecWrapper::Bool(val) => query.bind(val),
        };
    }

    let sections: Vec<SectionFromDb> = query.fetch_all(pool).await.unwrap();
    Ok(sections)
}

pub async fn create_section(
    pool: &sqlx::Pool<sqlx::MySql>,
    section_form: CreateSectionForm,
) -> Result<(), ()> {
    let next_pos = match get_max_position(pool).await {
        Some(num) => num + 1,
        None => 0,
    };
    let res = sqlx::query!(
        "INSERT INTO sections (title, position) VALUES (?, ?)",
        section_form.title,
        next_pos
    )
    .execute(pool)
    .await;
    match res {
        Ok(_) => Ok(()),
        Err(_) => Err(()),
    }
}

pub struct UpdateSectionForm {
    pub title: Option<String>,
}

pub enum UpdateSectionsError {
    UnexpectedError,
    NotFoundError,
}

pub async fn update_sections(
    pool: &sqlx::Pool<sqlx::MySql>,
    section_form: UpdateSectionForm,
    identified_by: GetSectionsForm,
) -> Result<(), UpdateSectionsError> {
    todo!()
}
