use crate::db;
use serde::Serialize;

pub struct CreateSubsectionForm {
    pub title: String,
    pub section_id: u32,
}

pub struct UpdateSubsectionForm {
    pub title: Option<String>,
    pub section_id: Option<u32>,
    pub position: Option<u32>,
}

pub struct GetSubsectionsForm {
    pub id: Option<u32>,
    pub title: Option<String>,
    pub position: Option<u32>,
    pub section_id: Option<u32>,
    pub limit: Option<u32>,
}

#[derive(Serialize)]
pub struct SubsectionReturn {
    pub id: u32,
    pub title: String,
    pub position: u32,
    pub section_id: u32,
}

impl From<db::subsections::SubsectionFromDb> for SubsectionReturn {
    fn from(value: db::subsections::SubsectionFromDb) -> Self {
        Self {
            id: value.id,
            title: value.title,
            position: value.position,
            section_id: value.section_id,
        }
    }
}

#[derive(Debug)]
pub enum CreateSubsectionError {
    UnexpectedError,
}

pub async fn create_subsection(
    pool: &sqlx::Pool<sqlx::MySql>,
    form: CreateSubsectionForm,
) -> Result<(), CreateSubsectionError> {
    db::subsections::create_subsection(
        pool,
        db::subsections::CreateSubsectionForm {
            title: form.title,
            section_id: form.section_id,
        },
    )
    .await
    .map_err(|_| CreateSubsectionError::UnexpectedError)
}

#[derive(Debug)]
pub enum GetSubsectionsError {
    UnexpectedError,
}

pub async fn get_subsections(
    pool: &sqlx::Pool<sqlx::MySql>,
    form: GetSubsectionsForm,
) -> Result<Vec<SubsectionReturn>, GetSubsectionsError> {
    let res = db::subsections::get_subsections(
        pool,
        db::subsections::GetSubsectionsForm {
            id: form.id,
            title: form.title,
            position: form.position,
            section_id: form.section_id,
            limit: form.limit,
            ..Default::default()
        },
    )
    .await;
    match res {
        Ok(list) => Ok(list.into_iter().map(SubsectionReturn::from).collect()),
        Err(db::subsections::GetSubsectionsError::UnexpectedError) => {
            Err(GetSubsectionsError::UnexpectedError)
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
    id: u32,
) -> Result<SubsectionReturn, GetSubsectionError> {
    let res = db::subsections::get_subsection(
        pool,
        db::subsections::GetSubsectionsForm {
            id: Some(id),
            ..Default::default()
        },
    )
    .await;
    match res {
        Ok(val) => Ok(SubsectionReturn::from(val)),
        Err(db::subsections::GetSubsectionError::NotFoundError) => {
            Err(GetSubsectionError::NotFoundError)
        }
        Err(db::subsections::GetSubsectionError::UnexpectedError) => {
            Err(GetSubsectionError::UnexpectedError)
        }
    }
}

#[derive(Debug)]
pub enum UpdateSubsectionError {
    UnexpectedError,
    NotFoundError,
    NothingToUpdateError,
}

pub async fn update_subsection(
    pool: &sqlx::Pool<sqlx::MySql>,
    id: u32,
    form: UpdateSubsectionForm,
) -> Result<(), UpdateSubsectionError> {
    let res = db::subsections::update_subsections(
        pool,
        db::subsections::UpdateSubsectionForm {
            title: form.title,
            section_id: form.section_id,
            position: form.position,
        },
        db::subsections::GetSubsectionsForm {
            id: Some(id),
            ..Default::default()
        },
    )
    .await;
    match res {
        Ok(()) => Ok(()),
        Err(db::subsections::UpdateSubsectionsError::NotFoundError) => {
            Err(UpdateSubsectionError::NotFoundError)
        }
        Err(db::subsections::UpdateSubsectionsError::NothingToUpdateError) => {
            Err(UpdateSubsectionError::NothingToUpdateError)
        }
        Err(db::subsections::UpdateSubsectionsError::UnexpectedError) => {
            Err(UpdateSubsectionError::UnexpectedError)
        }
    }
}

#[derive(Debug)]
pub enum DeleteSubsectionError {
    UnexpectedError,
}

pub async fn delete_subsection(
    pool: &sqlx::Pool<sqlx::MySql>,
    id: u32,
) -> Result<(), DeleteSubsectionError> {
    db::subsections::delete_subsection(
        pool,
        db::subsections::GetSubsectionsForm {
            id: Some(id),
            ..Default::default()
        },
    )
    .await
    .map_err(|_| DeleteSubsectionError::UnexpectedError)
}

#[derive(Debug)]
pub enum MoveSubsectionError {
    UnexpectedError,
    NotFoundError(Option<u32>, Option<u32>),
    CantSwapFromDifferentSections,
}

pub async fn move_subsection(
    pool: &sqlx::Pool<sqlx::MySql>,
    ids: [u32; 2],
) -> Result<(), MoveSubsectionError> {
    let res = db::subsections::swap_subsections(pool, ids).await;
    match res {
        Ok(()) => Ok(()),
        Err(db::subsections::SwapSubsectionsError::NotFoundError(tuple)) => {
            Err(MoveSubsectionError::NotFoundError(tuple.0, tuple.1))
        }
        Err(db::subsections::SwapSubsectionsError::CantSwapFromDifferentSections) => {
            Err(MoveSubsectionError::CantSwapFromDifferentSections)
        }
        Err(db::subsections::SwapSubsectionsError::UnexpectedError) => {
            Err(MoveSubsectionError::UnexpectedError)
        }
    }
}
