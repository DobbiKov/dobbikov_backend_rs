use crate::db;
use serde::Serialize;

/// Maximum length of a tag name (matches the `tags.name` column).
pub const MAX_TAG_NAME_LEN: usize = 64;

pub struct CreateTagForm {
    pub name: String,
    pub color: String,
}

pub struct UpdateTagForm {
    pub name: Option<String>,
    pub color: Option<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub struct TagReturn {
    pub id: u32,
    pub name: String,
    pub color: String,
}

impl From<db::tags::TagFromDb> for TagReturn {
    fn from(value: db::tags::TagFromDb) -> Self {
        Self {
            id: value.id,
            name: value.name,
            color: value.color,
        }
    }
}

/// Reasons a tag name or color is rejected.
#[derive(Debug, PartialEq, Eq)]
pub enum TagValidationError {
    EmptyName,
    NameTooLong,
    InvalidColor,
}

/// Trim the name and ensure it is non-empty and fits into the column.
pub fn normalize_tag_name(name: &str) -> Result<String, TagValidationError> {
    let name = name.trim();
    if name.is_empty() {
        return Err(TagValidationError::EmptyName);
    }
    if name.chars().count() > MAX_TAG_NAME_LEN {
        return Err(TagValidationError::NameTooLong);
    }
    Ok(name.to_string())
}

/// Accept `#rgb` or `#rrggbb` (case-insensitive) and return it as lowercase `#rrggbb`.
pub fn normalize_tag_color(color: &str) -> Result<String, TagValidationError> {
    let hex = color
        .trim()
        .strip_prefix('#')
        .ok_or(TagValidationError::InvalidColor)?;
    if !hex.chars().all(|ch| ch.is_ascii_hexdigit()) {
        return Err(TagValidationError::InvalidColor);
    }
    let hex = hex.to_ascii_lowercase();
    match hex.len() {
        6 => Ok(format!("#{}", hex)),
        3 => Ok(format!(
            "#{}",
            hex.chars().flat_map(|ch| [ch, ch]).collect::<String>()
        )),
        _ => Err(TagValidationError::InvalidColor),
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum CreateTagError {
    UnexpectedError,
    AlreadyExistsError,
    ValidationError(TagValidationError),
}

pub async fn create_tag(
    pool: &sqlx::Pool<sqlx::MySql>,
    form: CreateTagForm,
) -> Result<TagReturn, CreateTagError> {
    let name = normalize_tag_name(&form.name).map_err(CreateTagError::ValidationError)?;
    let color = normalize_tag_color(&form.color).map_err(CreateTagError::ValidationError)?;
    let id = db::tags::create_tag(
        pool,
        db::tags::CreateTagForm {
            name: name.clone(),
            color: color.clone(),
        },
    )
    .await
    .map_err(|err| match err {
        db::tags::CreateTagError::AlreadyExistsError => CreateTagError::AlreadyExistsError,
        db::tags::CreateTagError::UnexpectedError => CreateTagError::UnexpectedError,
    })?;
    Ok(TagReturn { id, name, color })
}

#[derive(Debug)]
pub enum GetTagsError {
    UnexpectedError,
}

pub async fn get_tags(pool: &sqlx::Pool<sqlx::MySql>) -> Result<Vec<TagReturn>, GetTagsError> {
    db::tags::get_tags(pool)
        .await
        .map(|list| list.into_iter().map(TagReturn::from).collect())
        .map_err(|_| GetTagsError::UnexpectedError)
}

#[derive(Debug)]
pub enum GetTagError {
    UnexpectedError,
    NotFoundError,
}

pub async fn get_tag(pool: &sqlx::Pool<sqlx::MySql>, id: u32) -> Result<TagReturn, GetTagError> {
    match db::tags::get_tag(pool, id).await {
        Ok(val) => Ok(TagReturn::from(val)),
        Err(db::tags::GetTagError::NotFoundError) => Err(GetTagError::NotFoundError),
        Err(db::tags::GetTagError::UnexpectedError) => Err(GetTagError::UnexpectedError),
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum UpdateTagError {
    UnexpectedError,
    NotFoundError,
    NothingToUpdateError,
    AlreadyExistsError,
    ValidationError(TagValidationError),
}

pub async fn update_tag(
    pool: &sqlx::Pool<sqlx::MySql>,
    id: u32,
    form: UpdateTagForm,
) -> Result<(), UpdateTagError> {
    let name = form
        .name
        .as_deref()
        .map(normalize_tag_name)
        .transpose()
        .map_err(UpdateTagError::ValidationError)?;
    let color = form
        .color
        .as_deref()
        .map(normalize_tag_color)
        .transpose()
        .map_err(UpdateTagError::ValidationError)?;
    let res = db::tags::update_tag(pool, id, db::tags::UpdateTagForm { name, color }).await;
    match res {
        Ok(()) => Ok(()),
        Err(db::tags::UpdateTagError::NotFoundError) => Err(UpdateTagError::NotFoundError),
        Err(db::tags::UpdateTagError::NothingToUpdateError) => {
            Err(UpdateTagError::NothingToUpdateError)
        }
        Err(db::tags::UpdateTagError::AlreadyExistsError) => {
            Err(UpdateTagError::AlreadyExistsError)
        }
        Err(db::tags::UpdateTagError::UnexpectedError) => Err(UpdateTagError::UnexpectedError),
    }
}

#[derive(Debug)]
pub enum DeleteTagError {
    UnexpectedError,
}

pub async fn delete_tag(pool: &sqlx::Pool<sqlx::MySql>, id: u32) -> Result<(), DeleteTagError> {
    db::tags::delete_tag(pool, id)
        .await
        .map_err(|_| DeleteTagError::UnexpectedError)
}

#[derive(Debug, PartialEq, Eq)]
pub enum AssignTagsError {
    UnexpectedError,
    NoteNotFoundError,
    TagsNotFoundError(Vec<u32>),
}

impl From<db::tags::AssignTagsError> for AssignTagsError {
    fn from(value: db::tags::AssignTagsError) -> Self {
        match value {
            db::tags::AssignTagsError::UnexpectedError => Self::UnexpectedError,
            db::tags::AssignTagsError::NoteNotFoundError => Self::NoteNotFoundError,
            db::tags::AssignTagsError::TagsNotFoundError(ids) => Self::TagsNotFoundError(ids),
        }
    }
}

pub async fn set_note_tags(
    pool: &sqlx::Pool<sqlx::MySql>,
    note_id: u32,
    tag_ids: &[u32],
) -> Result<(), AssignTagsError> {
    db::tags::set_note_tags(pool, note_id, tag_ids)
        .await
        .map_err(AssignTagsError::from)
}

pub async fn add_tag_to_note(
    pool: &sqlx::Pool<sqlx::MySql>,
    note_id: u32,
    tag_id: u32,
) -> Result<(), AssignTagsError> {
    db::tags::add_tag_to_note(pool, note_id, tag_id)
        .await
        .map_err(AssignTagsError::from)
}

pub async fn remove_tag_from_note(
    pool: &sqlx::Pool<sqlx::MySql>,
    note_id: u32,
    tag_id: u32,
) -> Result<(), AssignTagsError> {
    db::tags::remove_tag_from_note(pool, note_id, tag_id)
        .await
        .map_err(AssignTagsError::from)
}
