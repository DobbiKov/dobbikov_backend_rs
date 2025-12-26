use crate::db;
use serde::Serialize;

pub struct CreateNoteForm {
    pub name: String,
    pub url: String,
    pub section_id: Option<u32>,
    pub subsection_id: Option<u32>,
}

pub struct UpdateNoteForm {
    pub name: Option<String>,
    pub url: Option<String>,
    pub section_id: Option<u32>,
    pub subsection_id: Option<u32>,
    pub position: Option<u32>,
}

pub struct GetNotesForm {
    pub id: Option<u32>,
    pub name: Option<String>,
    pub url: Option<String>,
    pub position: Option<u32>,
    pub section_id: Option<u32>,
    pub subsection_id: Option<u32>,
    pub limit: Option<u32>,
}

#[derive(Serialize)]
pub struct NoteReturn {
    pub id: u32,
    pub name: String,
    pub url: String,
    pub position: u32,
    pub section_id: Option<u32>,
    pub subsection_id: Option<u32>,
}

impl From<db::lecture_notes::NoteFromDb> for NoteReturn {
    fn from(value: db::lecture_notes::NoteFromDb) -> Self {
        Self {
            id: value.id,
            name: value.name,
            url: value.url,
            position: value.position,
            section_id: value.section_id,
            subsection_id: value.subsection_id,
        }
    }
}

#[derive(Debug)]
pub enum CreateNoteError {
    UnexpectedError,
}

pub async fn create_note(
    pool: &sqlx::Pool<sqlx::MySql>,
    form: CreateNoteForm,
) -> Result<(), CreateNoteError> {
    db::lecture_notes::create_note(
        pool,
        db::lecture_notes::CreateNoteForm {
            name: form.name,
            url: form.url,
            section_id: form.section_id,
            subsection_id: form.subsection_id,
        },
    )
    .await
    .map_err(|_| CreateNoteError::UnexpectedError)
}

#[derive(Debug)]
pub enum GetNotesError {
    UnexpectedError,
}

pub async fn get_notes(
    pool: &sqlx::Pool<sqlx::MySql>,
    form: GetNotesForm,
) -> Result<Vec<NoteReturn>, GetNotesError> {
    let res = db::lecture_notes::get_notes(
        pool,
        db::lecture_notes::GetNotesForm {
            id: form.id,
            name: form.name,
            url: form.url,
            position: form.position,
            section_id: form.section_id,
            subsection_id: form.subsection_id,
            limit: form.limit,
            ..Default::default()
        },
    )
    .await;
    match res {
        Ok(list) => Ok(list.into_iter().map(NoteReturn::from).collect()),
        Err(db::lecture_notes::GetNotesError::UnexpectedError) => Err(GetNotesError::UnexpectedError),
    }
}

#[derive(Debug)]
pub enum GetNoteError {
    UnexpectedError,
    NotFoundError,
}

pub async fn get_note(pool: &sqlx::Pool<sqlx::MySql>, id: u32) -> Result<NoteReturn, GetNoteError> {
    let res = db::lecture_notes::get_note(
        pool,
        db::lecture_notes::GetNotesForm {
            id: Some(id),
            ..Default::default()
        },
    )
    .await;
    match res {
        Ok(val) => Ok(NoteReturn::from(val)),
        Err(db::lecture_notes::GetNoteError::NotFoundError) => Err(GetNoteError::NotFoundError),
        Err(db::lecture_notes::GetNoteError::UnexpectedError) => {
            Err(GetNoteError::UnexpectedError)
        }
    }
}

#[derive(Debug)]
pub enum UpdateNoteError {
    UnexpectedError,
    NotFoundError,
    NothingToUpdateError,
}

pub async fn update_note(
    pool: &sqlx::Pool<sqlx::MySql>,
    id: u32,
    form: UpdateNoteForm,
) -> Result<(), UpdateNoteError> {
    let res = db::lecture_notes::update_notes(
        pool,
        db::lecture_notes::UpdateNoteForm {
            name: form.name,
            url: form.url,
            section_id: form.section_id,
            subsection_id: form.subsection_id,
            position: form.position,
        },
        db::lecture_notes::GetNotesForm {
            id: Some(id),
            ..Default::default()
        },
    )
    .await;
    match res {
        Ok(()) => Ok(()),
        Err(db::lecture_notes::UpdateNotesError::NotFoundError) => {
            Err(UpdateNoteError::NotFoundError)
        }
        Err(db::lecture_notes::UpdateNotesError::NothingToUpdateError) => {
            Err(UpdateNoteError::NothingToUpdateError)
        }
        Err(db::lecture_notes::UpdateNotesError::UnexpectedError) => {
            Err(UpdateNoteError::UnexpectedError)
        }
    }
}

#[derive(Debug)]
pub enum DeleteNoteError {
    UnexpectedError,
}

pub async fn delete_note(
    pool: &sqlx::Pool<sqlx::MySql>,
    id: u32,
) -> Result<(), DeleteNoteError> {
    db::lecture_notes::delete_note(
        pool,
        db::lecture_notes::GetNotesForm {
            id: Some(id),
            ..Default::default()
        },
    )
    .await
    .map_err(|_| DeleteNoteError::UnexpectedError)
}

#[derive(Debug)]
pub enum MoveNoteError {
    UnexpectedError,
    NotFoundError(Option<u32>, Option<u32>),
    CantSwapFromDifferentSubsections,
}

pub async fn move_note(
    pool: &sqlx::Pool<sqlx::MySql>,
    ids: [u32; 2],
) -> Result<(), MoveNoteError> {
    let res = db::lecture_notes::swap_notes(pool, ids).await;
    match res {
        Ok(()) => Ok(()),
        Err(db::lecture_notes::SwapNotesError::NotFoundError(tuple)) => {
            Err(MoveNoteError::NotFoundError(tuple.0, tuple.1))
        }
        Err(db::lecture_notes::SwapNotesError::CantSwapFromDifferentSubsections) => {
            Err(MoveNoteError::CantSwapFromDifferentSubsections)
        }
        Err(db::lecture_notes::SwapNotesError::UnexpectedError) => {
            Err(MoveNoteError::UnexpectedError)
        }
    }
}
