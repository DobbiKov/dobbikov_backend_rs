use axum::middleware;
use axum::http::StatusCode;
use axum::routing::{get, options, post, put};
use axum::{response::Response, Router};
use serde::Serialize;

pub mod lecture_notes;
pub mod responses;
pub mod sections;
pub mod subsections;
pub mod users;

#[derive(Clone)]
pub struct AppState {
    pub pool: sqlx::Pool<sqlx::MySql>,
}

#[derive(Serialize)]
pub struct RootNote {
    pub id: u32,
    pub name: String,
    pub url: String,
    pub position: u32,
    pub section_id: Option<u32>,
    pub subsection_id: Option<u32>,
}

#[derive(Serialize)]
pub struct RootSubsection {
    pub id: u32,
    pub title: String,
    pub position: u32,
    pub section_id: u32,
    pub notes: Vec<RootNote>,
}

#[derive(Serialize)]
pub struct RootSection {
    pub id: u32,
    pub title: String,
    pub position: u32,
    pub subsections: Vec<RootSubsection>,
    pub notes: Vec<RootNote>,
}

#[derive(Serialize)]
pub struct RootResponse {
    pub sections: Vec<RootSection>,
}

async fn root_index(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Result<axum::Json<RootResponse>, Response> {
    let sections = crate::services::sections::get_sections(
        &state.pool,
        crate::services::sections::GetSectionsForm {
            id: None,
            title: None,
            position: None,
            limit: None,
        },
    )
    .await
    .map_err(|_| responses::error_response(StatusCode::INTERNAL_SERVER_ERROR, "failed to load"))?;

    let subsections = crate::services::subsections::get_subsections(
        &state.pool,
        crate::services::subsections::GetSubsectionsForm {
            id: None,
            title: None,
            position: None,
            section_id: None,
            limit: None,
        },
    )
    .await
    .map_err(|_| responses::error_response(StatusCode::INTERNAL_SERVER_ERROR, "failed to load"))?;

    let notes = crate::services::lecture_notes::get_notes(
        &state.pool,
        crate::services::lecture_notes::GetNotesForm {
            id: None,
            name: None,
            url: None,
            position: None,
            section_id: None,
            subsection_id: None,
            limit: None,
        },
    )
    .await
    .map_err(|_| responses::error_response(StatusCode::INTERNAL_SERVER_ERROR, "failed to load"))?;

    let mut subsections_by_section: std::collections::HashMap<u32, Vec<RootSubsection>> =
        std::collections::HashMap::new();
    let mut notes_by_subsection: std::collections::HashMap<u32, Vec<RootNote>> =
        std::collections::HashMap::new();
    let mut notes_by_section: std::collections::HashMap<u32, Vec<RootNote>> =
        std::collections::HashMap::new();

    for note in notes {
        let note = RootNote {
            id: note.id,
            name: note.name,
            url: note.url,
            position: note.position,
            section_id: note.section_id,
            subsection_id: note.subsection_id,
        };

        if let Some(sub_id) = note.subsection_id {
            notes_by_subsection.entry(sub_id).or_default().push(note);
        } else if let Some(sec_id) = note.section_id {
            notes_by_section.entry(sec_id).or_default().push(note);
        }
    }

    for subsection in subsections {
        let mut sub_notes = notes_by_subsection
            .remove(&subsection.id)
            .unwrap_or_default();
        sub_notes.sort_by_key(|note| note.position);

        let item = RootSubsection {
            id: subsection.id,
            title: subsection.title,
            position: subsection.position,
            section_id: subsection.section_id,
            notes: sub_notes,
        };

        subsections_by_section
            .entry(subsection.section_id)
            .or_default()
            .push(item);
    }

    let mut section_items: Vec<RootSection> = sections
        .into_iter()
        .map(|section| {
            let mut section_subs = subsections_by_section
                .remove(&section.id)
                .unwrap_or_default();
            section_subs.sort_by_key(|sub| sub.position);

            let mut section_notes = notes_by_section.remove(&section.id).unwrap_or_default();
            section_notes.sort_by_key(|note| note.position);

            RootSection {
                id: section.id,
                title: section.title,
                position: section.position,
                subsections: section_subs,
                notes: section_notes,
            }
        })
        .collect();

    section_items.sort_by_key(|section| section.position);

    Ok(axum::Json(RootResponse {
        sections: section_items,
    }))
}

async fn admin_guard(
    axum::extract::State(state): axum::extract::State<AppState>,
    req: axum::http::Request<axum::body::Body>,
    next: middleware::Next,
) -> Response {
    let headers = req.headers();
    let token = headers
        .get("authorization")
        .and_then(|val| val.to_str().ok())
        .and_then(|val| val.strip_prefix("Bearer "))
        .map(|val| val.to_string());

    let token = match token {
        Some(value) => value,
        None => return responses::error_response(StatusCode::UNAUTHORIZED, "missing token"),
    };

    let auth_res = crate::services::users::authenticate_admin_by_token(&state.pool, token).await;
    if auth_res.is_err() {
        return responses::error_response(StatusCode::FORBIDDEN, "admin access required");
    }

    next.run(req).await
}

pub fn router(state: AppState) -> Router {
    let public_routes = Router::new()
        .route("/", get(root_index))
        .route("/sections", get(sections::list_sections))
        .route("/sections/{id}", get(sections::get_section))
        .route("/subsections", get(subsections::list_subsections))
        .route("/subsections/{id}", get(subsections::get_subsection))
        .route("/notes", get(lecture_notes::list_notes))
        .route("/notes/{id}", get(lecture_notes::get_note))
        .route("/users/register", post(users::register))
        .route("/users/login", post(users::login));

    let admin_routes = Router::new()
        .route("/sections", post(sections::create_section))
        .route(
            "/sections/{id}",
            put(sections::update_section).delete(sections::delete_section),
        )
        .route("/sections/move", post(sections::move_section))
        .route("/subsections", post(subsections::create_subsection))
        .route(
            "/subsections/{id}",
            put(subsections::update_subsection).delete(subsections::delete_subsection),
        )
        .route("/subsections/move", post(subsections::move_subsection))
        .route("/notes", post(lecture_notes::create_note))
        .route(
            "/notes/{id}",
            put(lecture_notes::update_note).delete(lecture_notes::delete_note),
        )
        .route("/notes/move", post(lecture_notes::move_note))
        .route("/users", get(users::list_users))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            admin_guard,
        ));

    Router::new()
        .merge(public_routes)
        .merge(admin_routes)
        .route("/{*path}", options(|| async { StatusCode::NO_CONTENT }))
        .with_state(state)
}
