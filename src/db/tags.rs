use std::collections::HashMap;

use crate::db::VecWrapper;
use loggit::{trace, warn};

/// The form used to create a new tag.
pub struct CreateTagForm {
    pub name: String,
    /// Color in the `#rrggbb` format.
    pub color: String,
}

/// The tag record as stored in the database.
#[derive(sqlx::FromRow, Debug, PartialEq, Eq, Default, Clone)]
pub struct TagFromDb {
    pub id: u32,
    pub name: String,
    pub color: String,
}

/// A tag together with the id of a note it is assigned to.
#[derive(sqlx::FromRow, Debug)]
struct NoteTagFromDb {
    note_id: u32,
    id: u32,
    name: String,
    color: String,
}

fn is_duplicate_entry(err: &sqlx::Error) -> bool {
    match err {
        sqlx::Error::Database(db_err) => db_err.is_unique_violation(),
        _ => false,
    }
}

/// Errors that might occur when creating a tag.
#[derive(Debug, PartialEq, Eq)]
pub enum CreateTagError {
    UnexpectedError,
    AlreadyExistsError,
}

/// Create a new tag and return its id.
pub async fn create_tag(
    pool: &sqlx::Pool<sqlx::MySql>,
    form: CreateTagForm,
) -> Result<u32, CreateTagError> {
    let res = sqlx::query("INSERT INTO tags (name, color) VALUES (?, ?)")
        .bind(form.name)
        .bind(form.color)
        .execute(pool)
        .await;
    trace!("{:?}", res);
    match res {
        Ok(val) => Ok(val.last_insert_id() as u32),
        Err(err) if is_duplicate_entry(&err) => Err(CreateTagError::AlreadyExistsError),
        Err(err) => {
            warn!("{:?}", err);
            Err(CreateTagError::UnexpectedError)
        }
    }
}

/// Errors that might occur when fetching tags.
#[derive(Debug)]
pub enum GetTagsError {
    UnexpectedError,
}

/// Fetch all tags ordered by name.
pub async fn get_tags(pool: &sqlx::Pool<sqlx::MySql>) -> Result<Vec<TagFromDb>, GetTagsError> {
    sqlx::query_as::<_, TagFromDb>("SELECT id, name, color FROM tags ORDER BY name")
        .fetch_all(pool)
        .await
        .map_err(|err| {
            warn!("{:?}", err);
            GetTagsError::UnexpectedError
        })
}

/// Errors that might occur when fetching a single tag.
#[derive(Debug, PartialEq, Eq)]
pub enum GetTagError {
    UnexpectedError,
    NotFoundError,
}

/// Fetch a single tag by its id.
pub async fn get_tag(pool: &sqlx::Pool<sqlx::MySql>, id: u32) -> Result<TagFromDb, GetTagError> {
    let res = sqlx::query_as::<_, TagFromDb>("SELECT id, name, color FROM tags WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await;
    match res {
        Ok(Some(tag)) => Ok(tag),
        Ok(None) => Err(GetTagError::NotFoundError),
        Err(err) => {
            warn!("{:?}", err);
            Err(GetTagError::UnexpectedError)
        }
    }
}

/// The form used to update one or more fields of a tag.
pub struct UpdateTagForm {
    pub name: Option<String>,
    pub color: Option<String>,
}

impl UpdateTagForm {
    pub fn is_all_none(&self) -> bool {
        self.name.is_none() && self.color.is_none()
    }
}

/// Errors that might occur when updating a tag.
#[derive(Debug, PartialEq, Eq)]
pub enum UpdateTagError {
    UnexpectedError,
    NotFoundError,
    NothingToUpdateError,
    AlreadyExistsError,
}

/// Update a tag identified by its id.
pub async fn update_tag(
    pool: &sqlx::Pool<sqlx::MySql>,
    id: u32,
    form: UpdateTagForm,
) -> Result<(), UpdateTagError> {
    if form.is_all_none() {
        return Err(UpdateTagError::NothingToUpdateError);
    }
    match get_tag(pool, id).await {
        Ok(_) => {}
        Err(GetTagError::NotFoundError) => return Err(UpdateTagError::NotFoundError),
        Err(GetTagError::UnexpectedError) => return Err(UpdateTagError::UnexpectedError),
    }

    let mut update_columns: Vec<String> = Vec::new();
    let mut update_params: Vec<VecWrapper> = Vec::new();

    if let Some(name) = form.name {
        update_columns.push("name = ?".to_string());
        update_params.push(VecWrapper::String(name));
    }
    if let Some(color) = form.color {
        update_columns.push("color = ?".to_string());
        update_params.push(VecWrapper::String(color));
    }

    let query_str = format!("UPDATE tags SET {} WHERE id = ?", update_columns.join(", "));
    let mut query = sqlx::query(query_str.as_str());
    for param in update_params {
        query = match param {
            VecWrapper::String(val) => query.bind(val),
            VecWrapper::Num(val) => query.bind(val),
            VecWrapper::Bool(val) => query.bind(val),
        };
    }
    let res = query.bind(id).execute(pool).await;
    match res {
        Ok(_) => Ok(()),
        Err(err) if is_duplicate_entry(&err) => Err(UpdateTagError::AlreadyExistsError),
        Err(err) => {
            warn!("{:?}", err);
            Err(UpdateTagError::UnexpectedError)
        }
    }
}

/// Error type for deleting tags.
#[derive(Debug)]
pub enum DeleteTagError {
    UnexpectedError,
}

/// Delete a tag. Its assignments to notes are removed by the `ON DELETE CASCADE` constraint.
pub async fn delete_tag(pool: &sqlx::Pool<sqlx::MySql>, id: u32) -> Result<(), DeleteTagError> {
    sqlx::query("DELETE FROM tags WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await
        .map_err(|err| {
            warn!("{:?}", err);
            DeleteTagError::UnexpectedError
        })
        .map(|_| ())
}

/// Fetch the tags of the given notes, grouped by note id.
/// When `note_ids` is `None`, the tags of all notes are returned.
pub async fn get_tags_for_notes(
    pool: &sqlx::Pool<sqlx::MySql>,
    note_ids: Option<&[u32]>,
) -> Result<HashMap<u32, Vec<TagFromDb>>, GetTagsError> {
    let base_query = "SELECT nt.note_id, t.id, t.name, t.color FROM note_tags nt \
                      JOIN tags t ON t.id = nt.tag_id";
    let query_str = match note_ids {
        None => format!("{} ORDER BY t.name", base_query),
        Some([]) => return Ok(HashMap::new()),
        Some(ids) => format!(
            "{} WHERE nt.note_id IN ({}) ORDER BY t.name",
            base_query,
            vec!["?"; ids.len()].join(", ")
        ),
    };
    trace!("{}", query_str);
    let mut query = sqlx::query_as::<_, NoteTagFromDb>(query_str.as_str());
    for id in note_ids.unwrap_or_default() {
        query = query.bind(*id);
    }
    let rows = query.fetch_all(pool).await.map_err(|err| {
        warn!("{:?}", err);
        GetTagsError::UnexpectedError
    })?;

    let mut tags_by_note: HashMap<u32, Vec<TagFromDb>> = HashMap::new();
    for row in rows {
        tags_by_note
            .entry(row.note_id)
            .or_default()
            .push(TagFromDb {
                id: row.id,
                name: row.name,
                color: row.color,
            });
    }
    Ok(tags_by_note)
}

/// Errors that might occur when changing the tags assigned to a note.
#[derive(Debug, PartialEq, Eq)]
pub enum AssignTagsError {
    UnexpectedError,
    NoteNotFoundError,
    /// Contains the ids of the tags that do not exist.
    TagsNotFoundError(Vec<u32>),
}

/// Return the ids from `tag_ids` that do not correspond to an existing tag
/// (sorted, without duplicates).
pub async fn find_missing_tag_ids(
    pool: &sqlx::Pool<sqlx::MySql>,
    tag_ids: &[u32],
) -> Result<Vec<u32>, GetTagsError> {
    if tag_ids.is_empty() {
        return Ok(Vec::new());
    }
    let query_str = format!(
        "SELECT id FROM tags WHERE id IN ({})",
        vec!["?"; tag_ids.len()].join(", ")
    );
    let mut query = sqlx::query_scalar::<_, u32>(query_str.as_str());
    for id in tag_ids {
        query = query.bind(*id);
    }
    let existing = query.fetch_all(pool).await.map_err(|err| {
        warn!("{:?}", err);
        GetTagsError::UnexpectedError
    })?;
    let mut missing: Vec<u32> = tag_ids
        .iter()
        .copied()
        .filter(|id| !existing.contains(id))
        .collect();
    missing.sort_unstable();
    missing.dedup();
    Ok(missing)
}

async fn ensure_note_and_tags_exist(
    pool: &sqlx::Pool<sqlx::MySql>,
    note_id: u32,
    tag_ids: &[u32],
) -> Result<(), AssignTagsError> {
    let note_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM notes WHERE id = ?")
        .bind(note_id)
        .fetch_one(pool)
        .await
        .map_err(|err| {
            warn!("{:?}", err);
            AssignTagsError::UnexpectedError
        })?;
    if note_count == 0 {
        return Err(AssignTagsError::NoteNotFoundError);
    }
    let missing = find_missing_tag_ids(pool, tag_ids)
        .await
        .map_err(|_| AssignTagsError::UnexpectedError)?;
    if missing.is_empty() {
        Ok(())
    } else {
        Err(AssignTagsError::TagsNotFoundError(missing))
    }
}

/// Replace the set of tags assigned to a note with `tag_ids`.
/// An empty slice removes all tags from the note.
pub async fn set_note_tags(
    pool: &sqlx::Pool<sqlx::MySql>,
    note_id: u32,
    tag_ids: &[u32],
) -> Result<(), AssignTagsError> {
    ensure_note_and_tags_exist(pool, note_id, tag_ids).await?;

    let unexpected = |err: sqlx::Error| {
        warn!("{:?}", err);
        AssignTagsError::UnexpectedError
    };
    let mut tx = pool.begin().await.map_err(unexpected)?;
    sqlx::query("DELETE FROM note_tags WHERE note_id = ?")
        .bind(note_id)
        .execute(&mut *tx)
        .await
        .map_err(unexpected)?;
    for tag_id in tag_ids {
        sqlx::query("INSERT IGNORE INTO note_tags (note_id, tag_id) VALUES (?, ?)")
            .bind(note_id)
            .bind(*tag_id)
            .execute(&mut *tx)
            .await
            .map_err(unexpected)?;
    }
    tx.commit().await.map_err(unexpected)
}

/// Assign a single tag to a note. Assigning an already assigned tag is a no-op.
pub async fn add_tag_to_note(
    pool: &sqlx::Pool<sqlx::MySql>,
    note_id: u32,
    tag_id: u32,
) -> Result<(), AssignTagsError> {
    ensure_note_and_tags_exist(pool, note_id, &[tag_id]).await?;
    sqlx::query("INSERT IGNORE INTO note_tags (note_id, tag_id) VALUES (?, ?)")
        .bind(note_id)
        .bind(tag_id)
        .execute(pool)
        .await
        .map_err(|err| {
            warn!("{:?}", err);
            AssignTagsError::UnexpectedError
        })
        .map(|_| ())
}

/// Remove a single tag from a note. Removing a tag that is not assigned is a no-op.
pub async fn remove_tag_from_note(
    pool: &sqlx::Pool<sqlx::MySql>,
    note_id: u32,
    tag_id: u32,
) -> Result<(), AssignTagsError> {
    sqlx::query("DELETE FROM note_tags WHERE note_id = ? AND tag_id = ?")
        .bind(note_id)
        .bind(tag_id)
        .execute(pool)
        .await
        .map_err(|err| {
            warn!("{:?}", err);
            AssignTagsError::UnexpectedError
        })
        .map(|_| ())
}
