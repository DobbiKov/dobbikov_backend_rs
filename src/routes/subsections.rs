use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::Response;
use axum::Json;
use serde::Deserialize;

use crate::routes::responses::{error_response, MessageResponse};
use crate::routes::AppState;
use crate::services;

#[derive(Deserialize)]
pub struct CreateSubsectionRequest {
    pub title: String,
    pub section_id: u32,
}

#[derive(Deserialize)]
pub struct UpdateSubsectionRequest {
    pub title: Option<String>,
    pub section_id: Option<u32>,
    pub position: Option<u32>,
}

#[derive(Deserialize)]
pub struct SubsectionQuery {
    pub id: Option<u32>,
    pub title: Option<String>,
    pub position: Option<u32>,
    pub section_id: Option<u32>,
    pub limit: Option<u32>,
}

#[derive(Deserialize)]
pub struct MoveSubsectionRequest {
    pub first_id: u32,
    pub second_id: u32,
}

pub async fn create_subsection(
    State(state): State<AppState>,
    Json(payload): Json<CreateSubsectionRequest>,
) -> Result<(StatusCode, Json<MessageResponse>), Response> {
    services::subsections::create_subsection(
        &state.pool,
        services::subsections::CreateSubsectionForm {
            title: payload.title,
            section_id: payload.section_id,
        },
    )
    .await
    .map_err(|_| error_response(StatusCode::INTERNAL_SERVER_ERROR, "failed to create subsection"))?;
    Ok((
        StatusCode::CREATED,
        Json(MessageResponse {
            message: "created".to_string(),
        }),
    ))
}

pub async fn list_subsections(
    State(state): State<AppState>,
    Query(query): Query<SubsectionQuery>,
) -> Result<Json<Vec<services::subsections::SubsectionReturn>>, Response> {
    let subsections = services::subsections::get_subsections(
        &state.pool,
        services::subsections::GetSubsectionsForm {
            id: query.id,
            title: query.title,
            position: query.position,
            section_id: query.section_id,
            limit: query.limit,
        },
    )
    .await
    .map_err(|_| {
        error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "failed to fetch subsections",
        )
    })?;
    Ok(Json(subsections))
}

pub async fn get_subsection(
    State(state): State<AppState>,
    Path(id): Path<u32>,
) -> Result<Json<services::subsections::SubsectionReturn>, Response> {
    let subsection =
        services::subsections::get_subsection(&state.pool, id)
            .await
            .map_err(|err| match err {
                services::subsections::GetSubsectionError::NotFoundError => {
                    error_response(StatusCode::NOT_FOUND, "subsection not found")
                }
                services::subsections::GetSubsectionError::UnexpectedError => {
                    error_response(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "failed to fetch subsection",
                    )
                }
            })?;
    Ok(Json(subsection))
}

pub async fn update_subsection(
    State(state): State<AppState>,
    Path(id): Path<u32>,
    Json(payload): Json<UpdateSubsectionRequest>,
) -> Result<Json<MessageResponse>, Response> {
    services::subsections::update_subsection(
        &state.pool,
        id,
        services::subsections::UpdateSubsectionForm {
            title: payload.title,
            section_id: payload.section_id,
            position: payload.position,
        },
    )
    .await
    .map_err(|err| match err {
        services::subsections::UpdateSubsectionError::NotFoundError => {
            error_response(StatusCode::NOT_FOUND, "subsection not found")
        }
        services::subsections::UpdateSubsectionError::NothingToUpdateError => {
            error_response(StatusCode::BAD_REQUEST, "nothing to update")
        }
        services::subsections::UpdateSubsectionError::UnexpectedError => {
            error_response(StatusCode::INTERNAL_SERVER_ERROR, "failed to update subsection")
        }
    })?;
    Ok(Json(MessageResponse {
        message: "updated".to_string(),
    }))
}

pub async fn delete_subsection(
    State(state): State<AppState>,
    Path(id): Path<u32>,
) -> Result<Json<MessageResponse>, Response> {
    services::subsections::delete_subsection(&state.pool, id)
        .await
        .map_err(|_| {
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to delete subsection",
            )
        })?;
    Ok(Json(MessageResponse {
        message: "deleted".to_string(),
    }))
}

pub async fn move_subsection(
    State(state): State<AppState>,
    Json(payload): Json<MoveSubsectionRequest>,
) -> Result<Json<MessageResponse>, Response> {
    services::subsections::move_subsection(&state.pool, [payload.first_id, payload.second_id])
        .await
        .map_err(|err| match err {
            services::subsections::MoveSubsectionError::NotFoundError(_, _) => {
                error_response(StatusCode::NOT_FOUND, "subsection not found")
            }
            services::subsections::MoveSubsectionError::CantSwapFromDifferentSections => {
                error_response(
                    StatusCode::BAD_REQUEST,
                    "cannot swap subsections from different sections",
                )
            }
            services::subsections::MoveSubsectionError::UnexpectedError => {
                error_response(StatusCode::INTERNAL_SERVER_ERROR, "failed to move subsection")
            }
        })?;
    Ok(Json(MessageResponse {
        message: "moved".to_string(),
    }))
}
