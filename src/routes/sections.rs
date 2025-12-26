use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::Response;
use axum::Json;
use serde::Deserialize;

use crate::routes::responses::{error_response, MessageResponse};
use crate::routes::AppState;
use crate::services;

#[derive(Deserialize)]
pub struct CreateSectionRequest {
    pub title: String,
}

#[derive(Deserialize)]
pub struct UpdateSectionRequest {
    pub title: Option<String>,
}

#[derive(Deserialize)]
pub struct SectionQuery {
    pub id: Option<u32>,
    pub title: Option<String>,
    pub position: Option<u32>,
    pub limit: Option<u32>,
}

#[derive(Deserialize)]
pub struct MoveSectionRequest {
    pub first_id: u32,
    pub second_id: u32,
}

pub async fn create_section(
    State(state): State<AppState>,
    Json(payload): Json<CreateSectionRequest>,
) -> Result<(StatusCode, Json<MessageResponse>), Response> {
    services::sections::create_section(
        &state.pool,
        services::sections::CreateSectionForm { title: payload.title },
    )
    .await
    .map_err(|_| error_response(StatusCode::INTERNAL_SERVER_ERROR, "failed to create section"))?;
    Ok((
        StatusCode::CREATED,
        Json(MessageResponse {
            message: "created".to_string(),
        }),
    ))
}

pub async fn list_sections(
    State(state): State<AppState>,
    Query(query): Query<SectionQuery>,
) -> Result<Json<Vec<services::sections::SectionReturn>>, Response> {
    let sections = services::sections::get_sections(
        &state.pool,
        services::sections::GetSectionsForm {
            id: query.id,
            title: query.title,
            position: query.position,
            limit: query.limit,
        },
    )
    .await
    .map_err(|_| error_response(StatusCode::INTERNAL_SERVER_ERROR, "failed to fetch sections"))?;
    Ok(Json(sections))
}

pub async fn get_section(
    State(state): State<AppState>,
    Path(id): Path<u32>,
) -> Result<Json<services::sections::SectionReturn>, Response> {
    let section = services::sections::get_section(&state.pool, id).await.map_err(|err| match err {
        services::sections::GetSectionError::NotFoundError => {
            error_response(StatusCode::NOT_FOUND, "section not found")
        }
        services::sections::GetSectionError::UnexpectedError => {
            error_response(StatusCode::INTERNAL_SERVER_ERROR, "failed to fetch section")
        }
    })?;
    Ok(Json(section))
}

pub async fn update_section(
    State(state): State<AppState>,
    Path(id): Path<u32>,
    Json(payload): Json<UpdateSectionRequest>,
) -> Result<Json<MessageResponse>, Response> {
    services::sections::update_section(
        &state.pool,
        id,
        services::sections::UpdateSectionForm { title: payload.title },
    )
    .await
    .map_err(|err| match err {
        services::sections::UpdateSectionError::NotFoundError => {
            error_response(StatusCode::NOT_FOUND, "section not found")
        }
        services::sections::UpdateSectionError::NothingToUpdateError => {
            error_response(StatusCode::BAD_REQUEST, "nothing to update")
        }
        services::sections::UpdateSectionError::UnexpectedError => {
            error_response(StatusCode::INTERNAL_SERVER_ERROR, "failed to update section")
        }
    })?;
    Ok(Json(MessageResponse {
        message: "updated".to_string(),
    }))
}

pub async fn delete_section(
    State(state): State<AppState>,
    Path(id): Path<u32>,
) -> Result<Json<MessageResponse>, Response> {
    services::sections::delete_section(&state.pool, id)
        .await
        .map_err(|_| error_response(StatusCode::INTERNAL_SERVER_ERROR, "failed to delete section"))?;
    Ok(Json(MessageResponse {
        message: "deleted".to_string(),
    }))
}

pub async fn move_section(
    State(state): State<AppState>,
    Json(payload): Json<MoveSectionRequest>,
) -> Result<Json<MessageResponse>, Response> {
    services::sections::move_section(&state.pool, [payload.first_id, payload.second_id])
        .await
        .map_err(|err| match err {
            services::sections::MoveSectionError::NotFoundError(_, _) => {
                error_response(StatusCode::NOT_FOUND, "section not found")
            }
            services::sections::MoveSectionError::UnexpectedError => {
                error_response(StatusCode::INTERNAL_SERVER_ERROR, "failed to move section")
            }
        })?;
    Ok(Json(MessageResponse {
        message: "moved".to_string(),
    }))
}
