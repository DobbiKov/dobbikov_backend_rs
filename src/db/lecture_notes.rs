use crate::db::{OrAnd, VecWrapper};

/// The form used to create a new note.
pub struct CreateNoteForm {
    pub name: String,
    pub url: String,
    pub section_id: Option<u32>,
    pub subsection_id: Option<u32>,
}

/// The note record as stored in the database.
#[derive(sqlx::FromRow, Debug, PartialEq, Eq, Default)]
pub struct NoteFromDb {
    pub id: u32,
    pub name: String,
    pub url: String,
    pub position: u32,
    pub section_id: Option<u32>,
    pub subsection_id: Option<u32>,
}

/// Get the maximum note position in a given subsection.
/// If no note exists in that subsection, returns None.
pub async fn get_max_note_position_in_subsection(
    pool: &sqlx::Pool<sqlx::MySql>,
    subsection_id: u32,
) -> Option<u32> {
    let query_str = "SELECT MAX(position) FROM notes WHERE subsection_id = ?";
    let query = sqlx::query_scalar(query_str).bind(subsection_id);
    let max: Result<Option<u32>, sqlx::Error> = query.fetch_one(pool).await;
    max.unwrap_or(None)
}

/// Struct to filter/select notes.
#[derive(Clone, Debug)]
pub struct GetNotesForm {
    pub id: Option<u32>,
    pub name: Option<String>,
    pub url: Option<String>,
    pub position: Option<u32>,
    pub section_id: Option<u32>,
    pub subsection_id: Option<u32>,
    pub or_and: OrAnd,
    pub limit: Option<u32>,
}

impl GetNotesForm {
    fn to_conditions_params(&self) -> (Vec<String>, Vec<VecWrapper>) {
        let mut conditions: Vec<String> = Vec::new();
        let mut params: Vec<VecWrapper> = Vec::new();

        if let Some(id) = self.id {
            conditions.push("id = ?".to_string());
            params.push(VecWrapper::Num(id));
        }
        if let Some(name) = &self.name {
            conditions.push("name = ?".to_string());
            params.push(VecWrapper::String(name.clone()));
        }
        if let Some(url) = &self.url {
            conditions.push("url = ?".to_string());
            params.push(VecWrapper::String(url.clone()));
        }
        if let Some(position) = self.position {
            conditions.push("position = ?".to_string());
            params.push(VecWrapper::Num(position));
        }
        if let Some(section_id) = self.section_id {
            conditions.push("section_id = ?".to_string());
            params.push(VecWrapper::Num(section_id));
        }
        if let Some(subsection_id) = self.subsection_id {
            conditions.push("subsection_id = ?".to_string());
            params.push(VecWrapper::Num(subsection_id));
        }
        (conditions, params)
    }
}

impl Default for GetNotesForm {
    fn default() -> Self {
        Self {
            id: None,
            name: None,
            url: None,
            position: None,
            section_id: None,
            subsection_id: None,
            or_and: Default::default(),
            limit: None,
        }
    }
}

/// Errors that might occur when fetching notes.
#[derive(Debug)]
pub enum GetNotesError {
    UnexpectedError,
}

/// Fetch notes based on the filtering form.
pub async fn get_notes(
    pool: &sqlx::Pool<sqlx::MySql>,
    form: GetNotesForm,
) -> Result<Vec<NoteFromDb>, GetNotesError> {
    let (conditions, params) = form.to_conditions_params();
    let pre_query_str = format!(
        "SELECT * FROM notes {} {} {}",
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
    let mut query = sqlx::query_as::<_, NoteFromDb>(query_str);
    for param in params {
        query = match param {
            VecWrapper::String(val) => query.bind(val),
            VecWrapper::Num(val) => query.bind(val),
            VecWrapper::Bool(val) => query.bind(val),
        };
    }
    let notes: Vec<NoteFromDb> = query.fetch_all(pool).await.unwrap();
    Ok(notes)
}

/// Error type when trying to get a single note.
#[derive(Debug)]
pub enum GetNoteError {
    UnexpectedError,
    NotFoundError,
}

/// Get a single note (using LIMIT 1) based on the filtering form.
pub async fn get_note(
    pool: &sqlx::Pool<sqlx::MySql>,
    form: GetNotesForm,
) -> Result<NoteFromDb, GetNoteError> {
    let notes = get_notes(
        pool,
        GetNotesForm {
            limit: Some(1),
            ..form
        },
    )
    .await;
    match notes {
        Ok(mut list) => {
            if list.is_empty() {
                Err(GetNoteError::NotFoundError)
            } else {
                Ok(list.swap_remove(0))
            }
        }
        Err(_) => Err(GetNoteError::UnexpectedError),
    }
}

/// Create a new note. Determines the next note position:
/// if a subsection is provided, it uses that; otherwise, it uses a global max.
pub async fn create_note(
    pool: &sqlx::Pool<sqlx::MySql>,
    note_form: CreateNoteForm,
) -> Result<(), ()> {
    let next_pos = if let Some(subsec_id) = note_form.subsection_id {
        match get_max_note_position_in_subsection(pool, subsec_id).await {
            Some(num) => num + 1,
            None => 0,
        }
    } else {
        let query_str = "SELECT MAX(position) FROM notes";
        let query = sqlx::query_scalar(query_str);
        let max: Result<Option<u32>, sqlx::Error> = query.fetch_one(pool).await;
        match max.unwrap_or(None) {
            Some(num) => num + 1,
            None => 0,
        }
    };

    let res = sqlx::query!(
        "INSERT INTO notes (name, url, position, section_id, subsection_id) VALUES (?, ?, ?, ?, ?)",
        note_form.name,
        note_form.url,
        next_pos,
        note_form.section_id,
        note_form.subsection_id,
    )
    .execute(pool)
    .await;
    println!("{:?}", res);
    match res {
        Ok(_) => Ok(()),
        Err(_) => Err(()),
    }
}

/// The form used to update one or more fields of a note.
pub struct UpdateNoteForm {
    pub name: Option<String>,
    pub url: Option<String>,
    pub section_id: Option<u32>,
    pub subsection_id: Option<u32>,
    pub position: Option<u32>,
}

impl UpdateNoteForm {
    pub fn is_all_none(&self) -> bool {
        self.name.is_none()
            && self.url.is_none()
            && self.section_id.is_none()
            && self.subsection_id.is_none()
            && self.position.is_none()
    }
}

/// Errors that might occur when updating notes.
#[derive(Debug, PartialEq, Eq)]
pub enum UpdateNotesError {
    UnexpectedError,
    NotFoundError,
    NothingToUpdateError,
}

/// Update notes based on the filtering form (`identified_by`).
pub async fn update_notes(
    pool: &sqlx::Pool<sqlx::MySql>,
    note_form: UpdateNoteForm,
    identified_by: GetNotesForm,
) -> Result<(), UpdateNotesError> {
    if note_form.is_all_none() {
        return Err(UpdateNotesError::NothingToUpdateError);
    }
    let notes_q = get_notes(pool, identified_by.clone()).await;
    if notes_q.is_err() {
        return Err(UpdateNotesError::UnexpectedError);
    }
    if let Ok(notes) = notes_q {
        if notes.is_empty() {
            return Err(UpdateNotesError::NotFoundError);
        }
    }

    let mut update_columns: Vec<String> = Vec::new();
    let mut update_params: Vec<VecWrapper> = Vec::new();

    if let Some(name) = note_form.name {
        update_columns.push("name = ?".to_string());
        update_params.push(VecWrapper::String(name));
    }
    if let Some(url) = note_form.url {
        update_columns.push("url = ?".to_string());
        update_params.push(VecWrapper::String(url));
    }
    if let Some(section_id) = note_form.section_id {
        update_columns.push("section_id = ?".to_string());
        update_params.push(VecWrapper::Num(section_id));
    }
    if let Some(subsection_id) = note_form.subsection_id {
        update_columns.push("subsection_id = ?".to_string());
        update_params.push(VecWrapper::Num(subsection_id));
    }
    if let Some(position) = note_form.position {
        update_columns.push("position = ?".to_string());
        update_params.push(VecWrapper::Num(position));
    }

    let (conditions, params) = identified_by.to_conditions_params();

    let pre_query_str = format!(
        "UPDATE notes SET {} {} {}",
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
            Err(UpdateNotesError::UnexpectedError)
        }
    }
}

/// Error types for swapping two notes.
#[derive(Debug, PartialEq, Eq)]
pub enum SwapNotesError {
    NotFoundError((Option<u32>, Option<u32>)),
    UnexpectedError,
    CantSwapFromDifferentSubsections,
}

/// Swap the positions of two notes identified by their IDs.
/// If both notes have a non-null `subsection_id`, they must be the same.
pub async fn swap_notes(
    pool: &sqlx::Pool<sqlx::MySql>,
    ids: [u32; 2],
) -> Result<(), SwapNotesError> {
    let note_1 = get_note(
        pool,
        GetNotesForm {
            id: Some(ids[0]),
            ..Default::default()
        },
    )
    .await;
    let note_2 = get_note(
        pool,
        GetNotesForm {
            id: Some(ids[1]),
            ..Default::default()
        },
    )
    .await;
    let mut ret_err_nf = (None, None);
    if note_1.is_err() {
        ret_err_nf.0 = Some(ids[0]);
    }
    if note_2.is_err() {
        ret_err_nf.1 = Some(ids[1]);
    }
    if ret_err_nf.0.is_some() || ret_err_nf.1.is_some() {
        return Err(SwapNotesError::NotFoundError(ret_err_nf));
    }
    let note_1 = note_1.unwrap();
    let note_2 = note_2.unwrap();
    if note_1.subsection_id.is_some()
        && note_2.subsection_id.is_some()
        && note_1.subsection_id != note_2.subsection_id
    {
        return Err(SwapNotesError::CantSwapFromDifferentSubsections);
    }

    // Use a temporary position that is higher than any existing one.
    let global_max_query = "SELECT MAX(position) FROM notes";
    let query = sqlx::query_scalar::<_, u32>(global_max_query);
    let max: Result<u32, sqlx::Error> = query.fetch_one(pool).await;
    let max_pos = max.unwrap_or(0) + 1;

    let (note_1_id, note_1_pos) = (note_1.id, note_1.position);
    let (note_2_id, note_2_pos) = (note_2.id, note_2.position);

    let update_query = "UPDATE notes SET position = ? WHERE id = ?";

    let query = sqlx::query(update_query)
        .bind(max_pos)
        .bind(note_1_id)
        .execute(pool)
        .await;
    if query.is_err() {
        return Err(SwapNotesError::UnexpectedError);
    }
    let query = sqlx::query(update_query)
        .bind(note_1_pos)
        .bind(note_2_id)
        .execute(pool)
        .await;
    if query.is_err() {
        return Err(SwapNotesError::UnexpectedError);
    }
    let query = sqlx::query(update_query)
        .bind(note_2_pos)
        .bind(note_1_id)
        .execute(pool)
        .await;
    query
        .map_err(|err| {
            println!("{:?}", err);
            SwapNotesError::UnexpectedError
        })
        .map(|_| ())
}

/// Error type for deleting notes.
pub enum DeleteNotesError {
    UnexpectedError,
}

/// Delete notes based on a filtering form.
pub async fn delete_notes(
    pool: &sqlx::Pool<sqlx::MySql>,
    form: GetNotesForm,
) -> Result<(), DeleteNotesError> {
    let (conditions, params) = form.to_conditions_params();

    let pre_query_str = format!(
        "DELETE FROM notes {} {} {}",
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
    let mut query = sqlx::query(query_str);
    for param in params {
        query = match param {
            VecWrapper::String(val) => query.bind(val),
            VecWrapper::Num(val) => query.bind(val),
            VecWrapper::Bool(val) => query.bind(val),
        };
    }
    let res = query.execute(pool).await;
    res.map_err(|_| DeleteNotesError::UnexpectedError)
        .map(|_| ())
}

/// Delete a single note (using LIMIT 1).
pub async fn delete_note(
    pool: &sqlx::Pool<sqlx::MySql>,
    form: GetNotesForm,
) -> Result<(), DeleteNotesError> {
    let new_form = GetNotesForm {
        limit: Some(1),
        ..form
    };
    delete_notes(pool, new_form).await
}
