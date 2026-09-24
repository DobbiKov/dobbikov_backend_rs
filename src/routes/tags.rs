use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::Response;
use axum::Json;
use serde::Deserialize;

use crate::routes::responses::{error_response, tags_not_found_response, MessageResponse};
use crate::routes::AppState;
use crate::services;
use crate::services::tags::{AssignTagsError, TagValidationError};

#[derive(Deserialize)]
pub struct CreateTagRequest {
    pub name: String,
    pub color: String,
}

#[derive(Deserialize)]
pub struct UpdateTagRequest {
    pub name: Option<String>,
    pub color: Option<String>,
}

#[derive(Deserialize)]
pub struct SetNoteTagsRequest {
    pub tag_ids: Vec<u32>,
}

fn validation_error_response(err: TagValidationError) -> Response {
    let message = match err {
        TagValidationError::EmptyName => "tag name must not be empty".to_string(),
        TagValidationError::NameTooLong => format!(
            "tag name must be at most {} characters",
            services::tags::MAX_TAG_NAME_LEN
        ),
        TagValidationError::InvalidColor => {
            "tag color must be a hex color like #1e90ff or #abc".to_string()
        }
    };
    error_response(StatusCode::BAD_REQUEST, message)
}

fn assign_error_response(err: AssignTagsError) -> Response {
    match err {
        AssignTagsError::NoteNotFoundError => {
            error_response(StatusCode::NOT_FOUND, "note not found")
        }
        AssignTagsError::TagsNotFoundError(ids) => tags_not_found_response(&ids),
        AssignTagsError::UnexpectedError => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "failed to update note tags",
        ),
    }
}

pub async fn create_tag(
    State(state): State<AppState>,
    Json(payload): Json<CreateTagRequest>,
) -> Result<(StatusCode, Json<services::tags::TagReturn>), Response> {
    let tag = services::tags::create_tag(
        &state.pool,
        services::tags::CreateTagForm {
            name: payload.name,
            color: payload.color,
        },
    )
    .await
    .map_err(|err| match err {
        services::tags::CreateTagError::ValidationError(err) => validation_error_response(err),
        services::tags::CreateTagError::AlreadyExistsError => {
            error_response(StatusCode::CONFLICT, "tag with this name already exists")
        }
        services::tags::CreateTagError::UnexpectedError => {
            error_response(StatusCode::INTERNAL_SERVER_ERROR, "failed to create tag")
        }
    })?;
    Ok((StatusCode::CREATED, Json(tag)))
}

pub async fn list_tags(
    State(state): State<AppState>,
) -> Result<Json<Vec<services::tags::TagReturn>>, Response> {
    let tags = services::tags::get_tags(&state.pool)
        .await
        .map_err(|_| error_response(StatusCode::INTERNAL_SERVER_ERROR, "failed to fetch tags"))?;
    Ok(Json(tags))
}

pub async fn get_tag(
    State(state): State<AppState>,
    Path(id): Path<u32>,
) -> Result<Json<services::tags::TagReturn>, Response> {
    let tag = services::tags::get_tag(&state.pool, id)
        .await
        .map_err(|err| match err {
            services::tags::GetTagError::NotFoundError => {
                error_response(StatusCode::NOT_FOUND, "tag not found")
            }
            services::tags::GetTagError::UnexpectedError => {
                error_response(StatusCode::INTERNAL_SERVER_ERROR, "failed to fetch tag")
            }
        })?;
    Ok(Json(tag))
}

pub async fn update_tag(
    State(state): State<AppState>,
    Path(id): Path<u32>,
    Json(payload): Json<UpdateTagRequest>,
) -> Result<Json<MessageResponse>, Response> {
    services::tags::update_tag(
        &state.pool,
        id,
        services::tags::UpdateTagForm {
            name: payload.name,
            color: payload.color,
        },
    )
    .await
    .map_err(|err| match err {
        services::tags::UpdateTagError::ValidationError(err) => validation_error_response(err),
        services::tags::UpdateTagError::NotFoundError => {
            error_response(StatusCode::NOT_FOUND, "tag not found")
        }
        services::tags::UpdateTagError::NothingToUpdateError => {
            error_response(StatusCode::BAD_REQUEST, "nothing to update")
        }
        services::tags::UpdateTagError::AlreadyExistsError => {
            error_response(StatusCode::CONFLICT, "tag with this name already exists")
        }
        services::tags::UpdateTagError::UnexpectedError => {
            error_response(StatusCode::INTERNAL_SERVER_ERROR, "failed to update tag")
        }
    })?;
    Ok(Json(MessageResponse {
        message: "updated".to_string(),
    }))
}

pub async fn delete_tag(
    State(state): State<AppState>,
    Path(id): Path<u32>,
) -> Result<Json<MessageResponse>, Response> {
    services::tags::delete_tag(&state.pool, id)
        .await
        .map_err(|_| error_response(StatusCode::INTERNAL_SERVER_ERROR, "failed to delete tag"))?;
    Ok(Json(MessageResponse {
        message: "deleted".to_string(),
    }))
}

pub async fn set_note_tags(
    State(state): State<AppState>,
    Path(id): Path<u32>,
    Json(payload): Json<SetNoteTagsRequest>,
) -> Result<Json<MessageResponse>, Response> {
    services::tags::set_note_tags(&state.pool, id, &payload.tag_ids)
        .await
        .map_err(assign_error_response)?;
    Ok(Json(MessageResponse {
        message: "updated".to_string(),
    }))
}

pub async fn add_tag_to_note(
    State(state): State<AppState>,
    Path((id, tag_id)): Path<(u32, u32)>,
) -> Result<Json<MessageResponse>, Response> {
    services::tags::add_tag_to_note(&state.pool, id, tag_id)
        .await
        .map_err(assign_error_response)?;
    Ok(Json(MessageResponse {
        message: "added".to_string(),
    }))
}

pub async fn remove_tag_from_note(
    State(state): State<AppState>,
    Path((id, tag_id)): Path<(u32, u32)>,
) -> Result<Json<MessageResponse>, Response> {
    services::tags::remove_tag_from_note(&state.pool, id, tag_id)
        .await
        .map_err(assign_error_response)?;
    Ok(Json(MessageResponse {
        message: "removed".to_string(),
    }))
}
