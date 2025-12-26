use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::Response;
use axum::Json;
use serde::Deserialize;

use crate::routes::responses::{error_response, MessageResponse};
use crate::routes::AppState;
use crate::services;

#[derive(Deserialize)]
pub struct CreateNoteRequest {
    pub name: String,
    pub url: String,
    pub section_id: Option<u32>,
    pub subsection_id: Option<u32>,
}

#[derive(Deserialize)]
pub struct UpdateNoteRequest {
    pub name: Option<String>,
    pub url: Option<String>,
    pub section_id: Option<u32>,
    pub subsection_id: Option<u32>,
    pub position: Option<u32>,
}

#[derive(Deserialize)]
pub struct NoteQuery {
    pub id: Option<u32>,
    pub name: Option<String>,
    pub url: Option<String>,
    pub position: Option<u32>,
    pub section_id: Option<u32>,
    pub subsection_id: Option<u32>,
    pub limit: Option<u32>,
}

#[derive(Deserialize)]
pub struct MoveNoteRequest {
    pub first_id: u32,
    pub second_id: u32,
}

pub async fn create_note(
    State(state): State<AppState>,
    Json(payload): Json<CreateNoteRequest>,
) -> Result<(StatusCode, Json<MessageResponse>), Response> {
    services::lecture_notes::create_note(
        &state.pool,
        services::lecture_notes::CreateNoteForm {
            name: payload.name,
            url: payload.url,
            section_id: payload.section_id,
            subsection_id: payload.subsection_id,
        },
    )
    .await
    .map_err(|_| error_response(StatusCode::INTERNAL_SERVER_ERROR, "failed to create note"))?;
    Ok((
        StatusCode::CREATED,
        Json(MessageResponse {
            message: "created".to_string(),
        }),
    ))
}

pub async fn list_notes(
    State(state): State<AppState>,
    Query(query): Query<NoteQuery>,
) -> Result<Json<Vec<services::lecture_notes::NoteReturn>>, Response> {
    let notes = services::lecture_notes::get_notes(
        &state.pool,
        services::lecture_notes::GetNotesForm {
            id: query.id,
            name: query.name,
            url: query.url,
            position: query.position,
            section_id: query.section_id,
            subsection_id: query.subsection_id,
            limit: query.limit,
        },
    )
    .await
    .map_err(|_| error_response(StatusCode::INTERNAL_SERVER_ERROR, "failed to fetch notes"))?;
    Ok(Json(notes))
}

pub async fn get_note(
    State(state): State<AppState>,
    Path(id): Path<u32>,
) -> Result<Json<services::lecture_notes::NoteReturn>, Response> {
    let note = services::lecture_notes::get_note(&state.pool, id).await.map_err(|err| match err {
        services::lecture_notes::GetNoteError::NotFoundError => {
            error_response(StatusCode::NOT_FOUND, "note not found")
        }
        services::lecture_notes::GetNoteError::UnexpectedError => {
            error_response(StatusCode::INTERNAL_SERVER_ERROR, "failed to fetch note")
        }
    })?;
    Ok(Json(note))
}

pub async fn update_note(
    State(state): State<AppState>,
    Path(id): Path<u32>,
    Json(payload): Json<UpdateNoteRequest>,
) -> Result<Json<MessageResponse>, Response> {
    services::lecture_notes::update_note(
        &state.pool,
        id,
        services::lecture_notes::UpdateNoteForm {
            name: payload.name,
            url: payload.url,
            section_id: payload.section_id,
            subsection_id: payload.subsection_id,
            position: payload.position,
        },
    )
    .await
    .map_err(|err| match err {
        services::lecture_notes::UpdateNoteError::NotFoundError => {
            error_response(StatusCode::NOT_FOUND, "note not found")
        }
        services::lecture_notes::UpdateNoteError::NothingToUpdateError => {
            error_response(StatusCode::BAD_REQUEST, "nothing to update")
        }
        services::lecture_notes::UpdateNoteError::UnexpectedError => {
            error_response(StatusCode::INTERNAL_SERVER_ERROR, "failed to update note")
        }
    })?;
    Ok(Json(MessageResponse {
        message: "updated".to_string(),
    }))
}

pub async fn delete_note(
    State(state): State<AppState>,
    Path(id): Path<u32>,
) -> Result<Json<MessageResponse>, Response> {
    services::lecture_notes::delete_note(&state.pool, id)
        .await
        .map_err(|_| error_response(StatusCode::INTERNAL_SERVER_ERROR, "failed to delete note"))?;
    Ok(Json(MessageResponse {
        message: "deleted".to_string(),
    }))
}

pub async fn move_note(
    State(state): State<AppState>,
    Json(payload): Json<MoveNoteRequest>,
) -> Result<Json<MessageResponse>, Response> {
    services::lecture_notes::move_note(&state.pool, [payload.first_id, payload.second_id])
        .await
        .map_err(|err| match err {
            services::lecture_notes::MoveNoteError::NotFoundError(_, _) => {
                error_response(StatusCode::NOT_FOUND, "note not found")
            }
            services::lecture_notes::MoveNoteError::CantSwapFromDifferentSubsections => {
                error_response(
                    StatusCode::BAD_REQUEST,
                    "cannot swap notes from different subsections",
                )
            }
            services::lecture_notes::MoveNoteError::UnexpectedError => {
                error_response(StatusCode::INTERNAL_SERVER_ERROR, "failed to move note")
            }
        })?;
    Ok(Json(MessageResponse {
        message: "moved".to_string(),
    }))
}
