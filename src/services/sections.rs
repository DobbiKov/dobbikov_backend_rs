use crate::db;
use serde::Serialize;

pub struct CreateSectionForm {
    pub title: String,
}

pub struct UpdateSectionForm {
    pub title: Option<String>,
}

pub struct GetSectionsForm {
    pub id: Option<u32>,
    pub title: Option<String>,
    pub position: Option<u32>,
    pub limit: Option<u32>,
}

#[derive(Serialize)]
pub struct SectionReturn {
    pub id: u32,
    pub title: String,
    pub position: u32,
}

impl From<db::sections::SectionFromDb> for SectionReturn {
    fn from(value: db::sections::SectionFromDb) -> Self {
        Self {
            id: value.id,
            title: value.title,
            position: value.position,
        }
    }
}

#[derive(Debug)]
pub enum CreateSectionError {
    UnexpectedError,
}

pub async fn create_section(
    pool: &sqlx::Pool<sqlx::MySql>,
    form: CreateSectionForm,
) -> Result<(), CreateSectionError> {
    db::sections::create_section(
        pool,
        db::sections::CreateSectionForm {
            title: form.title,
        },
    )
    .await
    .map_err(|_| CreateSectionError::UnexpectedError)
}

#[derive(Debug)]
pub enum GetSectionsError {
    UnexpectedError,
}

pub async fn get_sections(
    pool: &sqlx::Pool<sqlx::MySql>,
    form: GetSectionsForm,
) -> Result<Vec<SectionReturn>, GetSectionsError> {
    let res = db::sections::get_sections(
        pool,
        db::sections::GetSectionsForm {
            id: form.id,
            title: form.title,
            position: form.position,
            limit: form.limit,
            ..Default::default()
        },
    )
    .await;
    match res {
        Ok(list) => Ok(list.into_iter().map(SectionReturn::from).collect()),
        Err(db::sections::GetSectionsError::UnexpectedError) => {
            Err(GetSectionsError::UnexpectedError)
        }
    }
}

#[derive(Debug)]
pub enum GetSectionError {
    UnexpectedError,
    NotFoundError,
}

pub async fn get_section(
    pool: &sqlx::Pool<sqlx::MySql>,
    id: u32,
) -> Result<SectionReturn, GetSectionError> {
    let res = db::sections::get_section(
        pool,
        db::sections::GetSectionsForm {
            id: Some(id),
            ..Default::default()
        },
    )
    .await;
    match res {
        Ok(val) => Ok(SectionReturn::from(val)),
        Err(db::sections::GetSectionError::NotFoundError) => Err(GetSectionError::NotFoundError),
        Err(db::sections::GetSectionError::UnexpectedError) => {
            Err(GetSectionError::UnexpectedError)
        }
    }
}

#[derive(Debug)]
pub enum UpdateSectionError {
    UnexpectedError,
    NotFoundError,
    NothingToUpdateError,
}

pub async fn update_section(
    pool: &sqlx::Pool<sqlx::MySql>,
    id: u32,
    form: UpdateSectionForm,
) -> Result<(), UpdateSectionError> {
    let res = db::sections::update_sections(
        pool,
        db::sections::UpdateSectionForm { title: form.title },
        db::sections::GetSectionsForm {
            id: Some(id),
            ..Default::default()
        },
    )
    .await;
    match res {
        Ok(()) => Ok(()),
        Err(db::sections::UpdateSectionsError::NotFoundError) => {
            Err(UpdateSectionError::NotFoundError)
        }
        Err(db::sections::UpdateSectionsError::NothingToUpdateError) => {
            Err(UpdateSectionError::NothingToUpdateError)
        }
        Err(db::sections::UpdateSectionsError::UnexpectedError) => {
            Err(UpdateSectionError::UnexpectedError)
        }
    }
}

#[derive(Debug)]
pub enum DeleteSectionError {
    UnexpectedError,
}

pub async fn delete_section(
    pool: &sqlx::Pool<sqlx::MySql>,
    id: u32,
) -> Result<(), DeleteSectionError> {
    db::sections::delete_section(
        pool,
        db::sections::GetSectionsForm {
            id: Some(id),
            ..Default::default()
        },
    )
    .await
    .map_err(|_| DeleteSectionError::UnexpectedError)
}

#[derive(Debug)]
pub enum MoveSectionError {
    UnexpectedError,
    NotFoundError(Option<u32>, Option<u32>),
}

pub async fn move_section(
    pool: &sqlx::Pool<sqlx::MySql>,
    ids: [u32; 2],
) -> Result<(), MoveSectionError> {
    let res = db::sections::swap_sections(pool, ids).await;
    match res {
        Ok(()) => Ok(()),
        Err(db::sections::SwapSectionsError::NotFoundError(tuple)) => {
            Err(MoveSectionError::NotFoundError(tuple.0, tuple.1))
        }
        Err(db::sections::SwapSectionsError::UnexpectedError) => {
            Err(MoveSectionError::UnexpectedError)
        }
    }
}
