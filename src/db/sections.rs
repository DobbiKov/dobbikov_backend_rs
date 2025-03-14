use std::fmt::format;

use crate::db::{OrAnd, VecWrapper};

pub struct CreateSectionForm {
    pub title: String,
}

#[derive(sqlx::FromRow, Debug, PartialEq, Eq, Default)]
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

#[derive(Clone)]
pub struct GetSectionsForm {
    pub id: Option<u32>,
    pub title: Option<String>,
    pub position: Option<u32>,
    pub or_and: OrAnd,
    pub limit: Option<u32>,
}

impl GetSectionsForm {
    fn to_conditions_params(&self) -> (Vec<String>, Vec<VecWrapper>) {
        let mut conditions: Vec<String> = Vec::new();
        let mut params: Vec<VecWrapper> = Vec::new();

        if self.id.is_some() {
            conditions.push("id = ?".to_string());
            params.push(VecWrapper::Num(self.id.unwrap()));
        }
        if let Some(title) = &self.title {
            conditions.push("title = ?".to_string());
            params.push(VecWrapper::String(title.clone()));
        }
        if self.position.is_some() {
            conditions.push("position = ?".to_string());
            params.push(VecWrapper::Num(self.position.unwrap()));
        }
        (conditions, params)
    }
}

impl Default for GetSectionsForm {
    fn default() -> Self {
        Self {
            id: Default::default(),
            title: Default::default(),
            position: Default::default(),
            or_and: Default::default(),
            limit: None,
        }
    }
}

pub enum GetSectionsError {
    UnexpectedError,
}

pub async fn get_sections(
    pool: &sqlx::Pool<sqlx::MySql>,
    form: GetSectionsForm,
) -> Result<Vec<SectionFromDb>, GetSectionsError> {
    let (conditions, params) = form.to_conditions_params();
    let pre_query_str = format!(
        "SELECT * FROM sections {} {} {}",
        if !conditions.is_empty() { "WHERE" } else { "" },
        conditions.join(match form.or_and {
            OrAnd::And => " AND ",
            OrAnd::Or => " OR ",
        }),
        match form.limit {
            None => "".to_string(),
            Some(val) => {
                format!("LIMIT {}", val)
            }
        }
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
impl UpdateSectionForm {
    pub fn is_all_none(&self) -> bool {
        self.title.is_none()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum UpdateSectionsError {
    UnexpectedError,
    NotFoundError,
    NothingToUpdateError,
}

pub async fn update_sections(
    pool: &sqlx::Pool<sqlx::MySql>,
    section_form: UpdateSectionForm,
    identified_by: GetSectionsForm,
) -> Result<(), UpdateSectionsError> {
    //verify if there's no such section to update
    if section_form.is_all_none() {
        return Err(UpdateSectionsError::NothingToUpdateError);
    }
    let sections_q = get_sections(pool, identified_by.clone()).await;
    if sections_q.is_err() {
        return Err(UpdateSectionsError::UnexpectedError);
    }
    if let Ok(res) = sections_q {
        if res.is_empty() {
            return Err(UpdateSectionsError::NotFoundError);
        }
    }

    //updating query
    let mut update_columns: Vec<String> = Vec::new();
    let mut update_params: Vec<VecWrapper> = Vec::new();
    let mut conditions: Vec<String> = Vec::new();
    let mut params: Vec<VecWrapper> = Vec::new();

    if section_form.title.is_some() {
        update_columns.push("title = ?".to_string());
        update_params.push(VecWrapper::String(section_form.title.unwrap()));
    }

    let (conditions, params) = identified_by.to_conditions_params();

    let pre_query_str = format!(
        "UPDATE sections SET {} {} {}",
        update_columns.join(", "),
        if conditions.is_empty() { "" } else { "WHERE" },
        conditions.join(match identified_by.or_and {
            OrAnd::And => " AND ",
            OrAnd::Or => " OR ",
        })
    );
    let query_str = pre_query_str.as_str();

    let mut query = sqlx::query(query_str);

    for param in update_params {
        query = match param {
            VecWrapper::String(val) => query.bind(val),
            VecWrapper::Num(val) => query.bind(val),
            VecWrapper::Bool(val) => query.bind(val),
        };
    }

    for param in params {
        query = match param {
            VecWrapper::String(val) => query.bind(val),
            VecWrapper::Num(val) => query.bind(val),
            VecWrapper::Bool(val) => query.bind(val),
        };
    }
    //println!("{}", query);
    let res = query.execute(pool).await;
    match res {
        Ok(_) => Ok(()),
        Err(e) => {
            println!("{:?}", e);
            Err(UpdateSectionsError::UnexpectedError)
        }
    }
}

pub enum GetSectionError {
    UnexpectedError,
    NotFoundError,
}

pub async fn get_section(
    pool: &sqlx::Pool<sqlx::MySql>,
    form: GetSectionsForm,
) -> Result<SectionFromDb, GetSectionError> {
    let sec = get_sections(
        pool,
        GetSectionsForm {
            limit: Some(1),
            ..form
        },
    )
    .await;
    match sec {
        Ok(mut val) => {
            if val.is_empty() {
                Err(GetSectionError::NotFoundError)
            } else {
                Ok(val.swap_remove(0))
            }
        }
        Err(err) => match err {
            GetSectionsError::UnexpectedError => Err(GetSectionError::UnexpectedError),
        },
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum SwapSectionsError {
    NotFoundError((Option<u32>, Option<u32>)),
    UnexpectedError,
}

pub async fn swap_sections(
    pool: &sqlx::Pool<sqlx::MySql>,
    ids: [u32; 2],
) -> Result<(), SwapSectionsError> {
    let section_1 = get_section(
        pool,
        GetSectionsForm {
            id: Some(ids[0]),
            ..Default::default()
        },
    )
    .await;
    let section_2 = get_section(
        pool,
        GetSectionsForm {
            id: Some(ids[1]),
            ..Default::default()
        },
    )
    .await;
    let mut ret_err_nf = (None, None);
    if section_1.is_err() {
        ret_err_nf = (Some(ids[0]), ret_err_nf.1);
    }
    if section_2.is_err() {
        ret_err_nf = (ret_err_nf.0, Some(ids[1]));
    }
    if ret_err_nf.0.is_some() || ret_err_nf.1.is_some() {
        return Err(SwapSectionsError::NotFoundError(ret_err_nf));
    }

    let max_pos = get_max_position(pool).await.unwrap_or_default() + 1;
    let section_1 = section_1.unwrap_or_default();
    let section_2 = section_2.unwrap_or_default();
    let (sec_1_id, sec_1_pos) = (section_1.id, section_1.position);
    let (sec_2_id, sec_2_pos) = (section_2.id, section_2.position);

    let pre_query_str = "\
    UPDATE sections \
    SET position = ? WHERE id =  ?\
        "
    .to_string();
    let query = sqlx::query(&pre_query_str)
        .bind(max_pos)
        .bind(sec_1_id)
        .execute(pool)
        .await;
    if query.is_err() {
        return Err(SwapSectionsError::UnexpectedError);
    }
    let query = sqlx::query(&pre_query_str)
        .bind(sec_1_pos)
        .bind(sec_2_id)
        .execute(pool)
        .await;
    if query.is_err() {
        return Err(SwapSectionsError::UnexpectedError);
    }
    let query = sqlx::query(&pre_query_str)
        .bind(sec_2_pos)
        .bind(sec_1_id)
        .execute(pool)
        .await;
    query
        .map_err(|err| {
            println!("{:?}", err);
            SwapSectionsError::UnexpectedError
        })
        .map(|_| ())
}

pub enum DeleteSectionsError {
    UnexpectedError,
}

pub async fn delete_sections(
    pool: &sqlx::Pool<sqlx::MySql>,
    form: GetSectionsForm,
) -> Result<(), DeleteSectionsError> {
    let (conditions, params) = form.to_conditions_params();

    let pre_query_str = format!(
        "DELETE FROM sections {} {} {}",
        if !conditions.is_empty() { "WHERE" } else { "" },
        conditions.join(match form.or_and {
            OrAnd::And => " AND ",
            OrAnd::Or => " OR ",
        }),
        match form.limit {
            None => "".to_string(),
            Some(val) => {
                format!("LIMIT {}", val)
            }
        }
    );

    let query_str = pre_query_str.as_str();
    println!("{}", query_str);
    let mut query = sqlx::query(query_str);

    for param in params {
        query = match param {
            VecWrapper::String(val) => query.bind(val),
            VecWrapper::Num(val) => query.bind(val),
            VecWrapper::Bool(val) => query.bind(val),
        };
    }

    let res = query.execute(pool).await;
    res.map_err(|_| DeleteSectionsError::UnexpectedError)
        .map(|_| ())
}

pub async fn delete_section(
    pool: &sqlx::Pool<sqlx::MySql>,
    form: GetSectionsForm,
) -> Result<(), DeleteSectionsError> {
    let new_form = GetSectionsForm {
        limit: Some(1),
        ..form
    };
    delete_sections(pool, new_form).await
}
