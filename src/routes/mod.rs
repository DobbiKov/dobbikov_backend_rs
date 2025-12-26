use axum::routing::{get, post, put};
use axum::Router;

pub mod lecture_notes;
pub mod responses;
pub mod sections;
pub mod subsections;

#[derive(Clone)]
pub struct AppState {
    pub pool: sqlx::Pool<sqlx::MySql>,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route(
            "/sections",
            post(sections::create_section).get(sections::list_sections),
        )
        .route("/sections/move", post(sections::move_section))
        .route(
            "/sections/:id",
            get(sections::get_section)
                .put(sections::update_section)
                .delete(sections::delete_section),
        )
        .route(
            "/subsections",
            post(subsections::create_subsection).get(subsections::list_subsections),
        )
        .route("/subsections/move", post(subsections::move_subsection))
        .route(
            "/subsections/:id",
            get(subsections::get_subsection)
                .put(subsections::update_subsection)
                .delete(subsections::delete_subsection),
        )
        .route(
            "/notes",
            post(lecture_notes::create_note).get(lecture_notes::list_notes),
        )
        .route("/notes/move", post(lecture_notes::move_note))
        .route(
            "/notes/:id",
            get(lecture_notes::get_note)
                .put(lecture_notes::update_note)
                .delete(lecture_notes::delete_note),
        )
        .with_state(state)
}
