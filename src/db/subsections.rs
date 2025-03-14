use std::fmt::format;

use crate::db::{OrAnd, VecWrapper};

use super::sections::GetSectionsForm;

pub struct CreateSubsectionForm {
    pub title: String,
    pub section_id: u32,
}

#[derive(sqlx::FromRow, Debug, PartialEq, Eq, Default)]
pub struct SubsectionFromDb {
    pub id: u32,
    pub title: String,
    pub position: u32,
    pub section_id: u32,
}

pub async fn get_max_subsection_position_in_section(
    pool: &sqlx::Pool<sqlx::MySql>,
    section_id: u32,
) -> Option<u32> {
    let pre_query_str = format!("SELECT MAX(position) FROM subsections WHERE section_id = ?");
    let query_str = pre_query_str.as_str();
    let query = sqlx::query_scalar(query_str).bind(section_id);

    let max: Result<Option<u32>, sqlx::Error> = query.fetch_one(pool).await;
    max.unwrap_or(None)
}

#[derive(Clone)]
pub struct GetSubsectionsForm {
    pub id: Option<u32>,
    pub title: Option<String>,
    pub position: Option<u32>,
    pub section_id: Option<u32>,
    pub or_and: OrAnd,
    pub limit: Option<u32>,
}

impl Default for GetSubsectionsForm {
    fn default() -> Self {
        Self {
            id: Default::default(),
            title: Default::default(),
            position: Default::default(),
            section_id: Default::default(),
            or_and: Default::default(),
            limit: None,
        }
    }
}

pub enum GetSubsectionsError {
    UnexpectedError,
}

pub async fn get_subsections(
    pool: &sqlx::Pool<sqlx::MySql>,
    form: GetSubsectionsForm,
) -> Result<Vec<SubsectionFromDb>, GetSubsectionsError> {
    let mut conditions: Vec<String> = Vec::new();
    let mut params: Vec<VecWrapper> = Vec::new();

    if let Some(id) = form.id {
        conditions.push("id = ?".to_string());
        params.push(VecWrapper::Num(id));
    }
    if let Some(title) = form.title {
        conditions.push("title = ?".to_string());
        params.push(VecWrapper::String(title));
    }
    if let Some(position) = form.position {
        conditions.push("position = ?".to_string());
        params.push(VecWrapper::Num(position));
    }
    if let Some(section_id) = form.section_id {
        conditions.push("section_id = ?".to_string());
        params.push(VecWrapper::Num(section_id));
    }

    let pre_query_str = format!(
        "SELECT * FROM subsections {} {} {}",
        if !conditions.is_empty() { "WHERE" } else { "" },
        conditions.join(match form.or_and {
            OrAnd::And => " AND ",
            OrAnd::Or => " OR ",
        }),
        match form.limit {
            None => "".to_string(),
            Some(val) => format!("LIMIT {}", val),
        }
    );
    let query_str = pre_query_str.as_str();
    println!("{}", query_str);
    let mut query = sqlx::query_as::<_, SubsectionFromDb>(query_str);

    for param in params {
        query = match param {
            VecWrapper::String(val) => query.bind(val),
            VecWrapper::Num(val) => query.bind(val),
            VecWrapper::Bool(val) => query.bind(val),
        };
    }

    let subsections: Vec<SubsectionFromDb> = query.fetch_all(pool).await.unwrap();
    Ok(subsections)
}

pub async fn create_subsection(
    pool: &sqlx::Pool<sqlx::MySql>,
    subsection_form: CreateSubsectionForm,
) -> Result<(), ()> {
    let next_pos =
        match get_max_subsection_position_in_section(pool, subsection_form.section_id).await {
            Some(num) => num + 1,
            None => 0,
        };

    let res = sqlx::query!(
        "INSERT INTO subsections (title, position, section_id) VALUES (?, ?, ?)",
        subsection_form.title,
        next_pos,
        subsection_form.section_id,
    )
    .execute(pool)
    .await;
    match res {
        Ok(_) => Ok(()),
        Err(_) => Err(()),
    }
}

pub struct UpdateSubsectionForm {
    pub title: Option<String>,
    pub section_id: Option<u32>,
    pub position: Option<u32>,
}

impl UpdateSubsectionForm {
    pub fn is_all_none(&self) -> bool {
        self.title.is_none() && self.section_id.is_none()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum UpdateSubsectionsError {
    UnexpectedError,
    NotFoundError,
    NothingToUpdateError,
}

pub async fn update_subsections(
    pool: &sqlx::Pool<sqlx::MySql>,
    subsection_form: UpdateSubsectionForm,
    identified_by: GetSubsectionsForm,
) -> Result<(), UpdateSubsectionsError> {
    if subsection_form.is_all_none() {
        return Err(UpdateSubsectionsError::NothingToUpdateError);
    }
    let subsections_q = get_subsections(pool, identified_by.clone()).await;
    if subsections_q.is_err() {
        return Err(UpdateSubsectionsError::UnexpectedError);
    }
    if let Ok(res) = subsections_q {
        if res.is_empty() {
            return Err(UpdateSubsectionsError::NotFoundError);
        }
    }

    let mut update_columns: Vec<String> = Vec::new();
    let mut update_params: Vec<VecWrapper> = Vec::new();
    let mut conditions: Vec<String> = Vec::new();
    let mut params: Vec<VecWrapper> = Vec::new();

    if let Some(title) = subsection_form.title {
        update_columns.push("title = ?".to_string());
        update_params.push(VecWrapper::String(title));
    }
    if let Some(section_id) = subsection_form.section_id {
        update_columns.push("section_id = ?".to_string());
        update_params.push(VecWrapper::Num(section_id));
    }
    if let Some(position) = subsection_form.position {
        update_columns.push("position = ?".to_string());
        update_params.push(VecWrapper::Num(position));
    }

    if let Some(id) = identified_by.id {
        conditions.push("id = ?".to_string());
        params.push(VecWrapper::Num(id));
    }
    if let Some(title) = identified_by.title {
        conditions.push("title = ?".to_string());
        params.push(VecWrapper::String(title));
    }
    if let Some(position) = identified_by.position {
        conditions.push("position = ?".to_string());
        params.push(VecWrapper::Num(position));
    }
    if let Some(section_id) = identified_by.section_id {
        conditions.push("section_id = ?".to_string());
        params.push(VecWrapper::Num(section_id));
    }

    let pre_query_str = format!(
        "UPDATE subsections SET {} {} {}",
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

    let res = query.execute(pool).await;
    match res {
        Ok(_) => Ok(()),
        Err(e) => {
            println!("{:?}", e);
            Err(UpdateSubsectionsError::UnexpectedError)
        }
    }
}

#[derive(Debug)]
pub enum GetSubsectionError {
    UnexpectedError,
    NotFoundError,
}

pub async fn get_subsection(
    pool: &sqlx::Pool<sqlx::MySql>,
    form: GetSubsectionsForm,
) -> Result<SubsectionFromDb, GetSubsectionError> {
    let sub = get_subsections(
        pool,
        GetSubsectionsForm {
            limit: Some(1),
            ..form
        },
    )
    .await;
    match sub {
        Ok(mut val) => {
            if val.is_empty() {
                Err(GetSubsectionError::NotFoundError)
            } else {
                Ok(val.swap_remove(0))
            }
        }
        Err(_) => Err(GetSubsectionError::UnexpectedError),
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum SwapSubsectionsError {
    NotFoundError((Option<u32>, Option<u32>)),
    UnexpectedError,
    CantSwapFromDifferentSections,
}

pub async fn swap_subsections(
    pool: &sqlx::Pool<sqlx::MySql>,
    ids: [u32; 2],
) -> Result<(), SwapSubsectionsError> {
    let subsection_1 = get_subsection(
        pool,
        GetSubsectionsForm {
            id: Some(ids[0]),
            ..Default::default()
        },
    )
    .await;
    let subsection_2 = get_subsection(
        pool,
        GetSubsectionsForm {
            id: Some(ids[1]),
            ..Default::default()
        },
    )
    .await;
    let mut ret_err_nf = (None, None);
    if subsection_1.is_err() {
        ret_err_nf = (Some(ids[0]), ret_err_nf.1);
    }
    if subsection_2.is_err() {
        ret_err_nf = (ret_err_nf.0, Some(ids[1]));
    }
    if ret_err_nf.0.is_some() || ret_err_nf.1.is_some() {
        return Err(SwapSubsectionsError::NotFoundError(ret_err_nf));
    }

    let subsection_1 = subsection_1.unwrap();
    let subsection_2 = subsection_2.unwrap();
    if subsection_1.section_id != subsection_2.section_id {
        return Err(SwapSubsectionsError::CantSwapFromDifferentSections);
    }

    let max_pos = get_max_subsection_position_in_section(pool, subsection_1.section_id)
        .await
        .unwrap_or_default()
        + 1;
    let (sub_1_id, sub_1_pos) = (subsection_1.id, subsection_1.position);
    let (sub_2_id, sub_2_pos) = (subsection_2.id, subsection_2.position);

    let pre_query_str = "UPDATE subsections SET position = ? WHERE id = ?";
    let query = sqlx::query(pre_query_str)
        .bind(max_pos)
        .bind(sub_1_id)
        .execute(pool)
        .await;
    if query.is_err() {
        return Err(SwapSubsectionsError::UnexpectedError);
    }
    let query = sqlx::query(pre_query_str)
        .bind(sub_1_pos)
        .bind(sub_2_id)
        .execute(pool)
        .await;
    if query.is_err() {
        return Err(SwapSubsectionsError::UnexpectedError);
    }
    let query = sqlx::query(pre_query_str)
        .bind(sub_2_pos)
        .bind(sub_1_id)
        .execute(pool)
        .await;
    query
        .map_err(|err| {
            println!("{:?}", err);
            SwapSubsectionsError::UnexpectedError
        })
        .map(|_| ())
}

// deleting
pub enum DeleteSubsectionsError {
    UnexpectedError,
}

pub async fn delete_subsections(
    pool: &sqlx::Pool<sqlx::MySql>,
    form: GetSubsectionsForm,
) -> Result<(), DeleteSubsectionsError> {
    let mut conditions: Vec<String> = Vec::new();
    let mut params: Vec<VecWrapper> = Vec::new();

    if let Some(id) = form.id {
        conditions.push("id = ?".to_string());
        params.push(VecWrapper::Num(id));
    }
    if let Some(title) = form.title {
        conditions.push("title = ?".to_string());
        params.push(VecWrapper::String(title));
    }
    if let Some(position) = form.position {
        conditions.push("position = ?".to_string());
        params.push(VecWrapper::Num(position));
    }
    if let Some(section_id) = form.section_id {
        conditions.push("section_id = ?".to_string());
        params.push(VecWrapper::Num(section_id));
    }

    let pre_query_str = format!(
        "DELETE FROM subsections {} {} {}",
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

pub async fn delete_subsection(
    pool: &sqlx::Pool<sqlx::MySql>,
    form: GetSubsectionsForm,
) -> Result<(), DeleteSubsectionsError> {
    let new_form = GetSubsectionsForm {
        limit: Some(1),
        ..form
    };
    delete_subsections(pool, new_form).await
}
