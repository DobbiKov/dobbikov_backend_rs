use crate::db;
use crate::services::tags::TagReturn;
use serde::Serialize;

pub struct CreateNoteForm {
    pub name: String,
    pub description: String,
    pub url: String,
    pub section_id: Option<u32>,
    pub subsection_id: Option<u32>,
    /// Tags to assign to the newly created note.
    pub tag_ids: Vec<u32>,
}

pub struct UpdateNoteForm {
    pub name: Option<String>,
    pub description: Option<String>,
    pub url: Option<String>,
    pub section_id: Option<u32>,
    pub subsection_id: Option<u32>,
    pub position: Option<u32>,
    /// When set, replaces the tags assigned to the note.
    pub tag_ids: Option<Vec<u32>>,
}

pub struct GetNotesForm {
    pub id: Option<u32>,
    pub name: Option<String>,
    pub url: Option<String>,
    pub position: Option<u32>,
    pub section_id: Option<u32>,
    pub subsection_id: Option<u32>,
    pub tag_id: Option<u32>,
    pub limit: Option<u32>,
}

#[derive(Serialize)]
pub struct NoteReturn {
    pub id: u32,
    pub name: String,
    pub description: String,
    pub url: String,
    pub position: u32,
    pub section_id: Option<u32>,
    pub subsection_id: Option<u32>,
    pub tags: Vec<TagReturn>,
}

impl From<db::lecture_notes::NoteFromDb> for NoteReturn {
    fn from(value: db::lecture_notes::NoteFromDb) -> Self {
        Self {
            id: value.id,
            name: value.name,
            description: value.description,
            url: value.url,
            position: value.position,
            section_id: value.section_id,
            subsection_id: value.subsection_id,
            tags: Vec::new(),
        }
    }
}

impl NoteReturn {
    fn with_tags(value: db::lecture_notes::NoteFromDb, tags: Vec<db::tags::TagFromDb>) -> Self {
        Self {
            tags: tags.into_iter().map(TagReturn::from).collect(),
            ..Self::from(value)
        }
    }
}

#[derive(Debug)]
pub enum CreateNoteError {
    UnexpectedError,
    TagsNotFoundError(Vec<u32>),
}

pub async fn create_note(
    pool: &sqlx::Pool<sqlx::MySql>,
    form: CreateNoteForm,
) -> Result<(), CreateNoteError> {
    let missing = db::tags::find_missing_tag_ids(pool, &form.tag_ids)
        .await
        .map_err(|_| CreateNoteError::UnexpectedError)?;
    if !missing.is_empty() {
        return Err(CreateNoteError::TagsNotFoundError(missing));
    }
    let note_id = db::lecture_notes::create_note(
        pool,
        db::lecture_notes::CreateNoteForm {
            name: form.name,
            description: form.description,
            url: form.url,
            section_id: form.section_id,
            subsection_id: form.subsection_id,
        },
    )
    .await
    .map_err(|_| CreateNoteError::UnexpectedError)?;
    if form.tag_ids.is_empty() {
        return Ok(());
    }
    db::tags::set_note_tags(pool, note_id, &form.tag_ids)
        .await
        .map_err(|err| match err {
            db::tags::AssignTagsError::TagsNotFoundError(ids) => {
                CreateNoteError::TagsNotFoundError(ids)
            }
            _ => CreateNoteError::UnexpectedError,
        })
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
            tag_id: form.tag_id,
            limit: form.limit,
            ..Default::default()
        },
    )
    .await;
    let notes = match res {
        Ok(list) => list,
        Err(db::lecture_notes::GetNotesError::UnexpectedError) => {
            return Err(GetNotesError::UnexpectedError)
        }
    };
    let note_ids: Vec<u32> = notes.iter().map(|note| note.id).collect();
    let mut tags_by_note = db::tags::get_tags_for_notes(pool, Some(&note_ids))
        .await
        .map_err(|_| GetNotesError::UnexpectedError)?;
    Ok(notes
        .into_iter()
        .map(|note| {
            let tags = tags_by_note.remove(&note.id).unwrap_or_default();
            NoteReturn::with_tags(note, tags)
        })
        .collect())
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
    let note = match res {
        Ok(val) => val,
        Err(db::lecture_notes::GetNoteError::NotFoundError) => {
            return Err(GetNoteError::NotFoundError)
        }
        Err(db::lecture_notes::GetNoteError::UnexpectedError) => {
            return Err(GetNoteError::UnexpectedError)
        }
    };
    let tags = db::tags::get_tags_for_notes(pool, Some(&[note.id]))
        .await
        .map_err(|_| GetNoteError::UnexpectedError)?
        .remove(&note.id)
        .unwrap_or_default();
    Ok(NoteReturn::with_tags(note, tags))
}

#[derive(Debug)]
pub enum UpdateNoteError {
    UnexpectedError,
    NotFoundError,
    NothingToUpdateError,
    TagsNotFoundError(Vec<u32>),
}

pub async fn update_note(
    pool: &sqlx::Pool<sqlx::MySql>,
    id: u32,
    form: UpdateNoteForm,
) -> Result<(), UpdateNoteError> {
    let fields_form = db::lecture_notes::UpdateNoteForm {
        name: form.name,
        description: form.description,
        url: form.url,
        section_id: form.section_id,
        subsection_id: form.subsection_id,
        position: form.position,
    };

    // Only the tags are being changed: `update_notes` would report nothing to update.
    if fields_form.is_all_none() {
        return match form.tag_ids {
            None => Err(UpdateNoteError::NothingToUpdateError),
            Some(tag_ids) => set_tags_after_update(pool, id, &tag_ids).await,
        };
    }

    // Check the tags up front so that an invalid tag id does not leave a half-applied update.
    if let Some(tag_ids) = &form.tag_ids {
        let missing = db::tags::find_missing_tag_ids(pool, tag_ids)
            .await
            .map_err(|_| UpdateNoteError::UnexpectedError)?;
        if !missing.is_empty() {
            return Err(UpdateNoteError::TagsNotFoundError(missing));
        }
    }

    let res = db::lecture_notes::update_notes(
        pool,
        fields_form,
        db::lecture_notes::GetNotesForm {
            id: Some(id),
            ..Default::default()
        },
    )
    .await;
    match res {
        Ok(()) => match form.tag_ids {
            None => Ok(()),
            Some(tag_ids) => set_tags_after_update(pool, id, &tag_ids).await,
        },
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

async fn set_tags_after_update(
    pool: &sqlx::Pool<sqlx::MySql>,
    id: u32,
    tag_ids: &[u32],
) -> Result<(), UpdateNoteError> {
    db::tags::set_note_tags(pool, id, tag_ids)
        .await
        .map_err(|err| match err {
            db::tags::AssignTagsError::NoteNotFoundError => UpdateNoteError::NotFoundError,
            db::tags::AssignTagsError::TagsNotFoundError(ids) => {
                UpdateNoteError::TagsNotFoundError(ids)
            }
            db::tags::AssignTagsError::UnexpectedError => UpdateNoteError::UnexpectedError,
        })
}

#[derive(Debug)]
pub enum DeleteNoteError {
    UnexpectedError,
}

pub async fn delete_note(pool: &sqlx::Pool<sqlx::MySql>, id: u32) -> Result<(), DeleteNoteError> {
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

pub async fn move_note(pool: &sqlx::Pool<sqlx::MySql>, ids: [u32; 2]) -> Result<(), MoveNoteError> {
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
