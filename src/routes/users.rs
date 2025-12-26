use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::Response;
use axum::Json;
use serde::Deserialize;

use crate::routes::responses::error_response;
use crate::routes::AppState;
use crate::services;

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
    pub is_admin: bool,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct UsersQuery {
    pub id: Option<u32>,
    pub username: Option<String>,
    pub limit: Option<u32>,
}

pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<services::users::AuthResponse>), Response> {
    let auth = services::users::register(
        &state.pool,
        services::users::RegisterForm {
            username: payload.username,
            password: payload.password,
            is_admin: payload.is_admin,
        },
    )
    .await
    .map_err(|_| error_response(StatusCode::INTERNAL_SERVER_ERROR, "failed to register user"))?;
    Ok((StatusCode::CREATED, Json(auth)))
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<services::users::AuthResponse>, Response> {
    let auth = services::users::login(
        &state.pool,
        services::users::LoginForm {
            username: payload.username,
            password: payload.password,
        },
    )
    .await
    .map_err(|err| match err {
        services::users::LoginError::NotFoundError => {
            error_response(StatusCode::NOT_FOUND, "user not found")
        }
        services::users::LoginError::InvalidPassword => {
            error_response(StatusCode::UNAUTHORIZED, "invalid password")
        }
        services::users::LoginError::UnexpectedError => {
            error_response(StatusCode::INTERNAL_SERVER_ERROR, "failed to login")
        }
    })?;
    Ok(Json(auth))
}

pub async fn list_users(
    State(state): State<AppState>,
    Query(query): Query<UsersQuery>,
) -> Result<Json<Vec<services::users::UserReturn>>, Response> {
    let users = services::users::get_users(
        &state.pool,
        services::users::GetUsersForm {
            id: query.id,
            username: query.username,
            limit: query.limit,
        },
    )
    .await
    .map_err(|_| error_response(StatusCode::INTERNAL_SERVER_ERROR, "failed to fetch users"))?;
    Ok(Json(users))
}
