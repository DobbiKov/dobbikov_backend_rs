use axum::middleware;
use axum::routing::{get, post, put};
use axum::{http::StatusCode, response::Response, Router};

pub mod lecture_notes;
pub mod responses;
pub mod sections;
pub mod subsections;
pub mod users;

#[derive(Clone)]
pub struct AppState {
    pub pool: sqlx::Pool<sqlx::MySql>,
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
        .route("/sections", get(sections::list_sections))
        .route("/sections/:id", get(sections::get_section))
        .route("/subsections", get(subsections::list_subsections))
        .route("/subsections/:id", get(subsections::get_subsection))
        .route("/notes", get(lecture_notes::list_notes))
        .route("/notes/:id", get(lecture_notes::get_note))
        .route("/users/register", post(users::register))
        .route("/users/login", post(users::login));

    let admin_routes = Router::new()
        .route("/sections", post(sections::create_section))
        .route("/sections/:id", put(sections::update_section).delete(sections::delete_section))
        .route("/sections/move", post(sections::move_section))
        .route("/subsections", post(subsections::create_subsection))
        .route(
            "/subsections/:id",
            put(subsections::update_subsection).delete(subsections::delete_subsection),
        )
        .route("/subsections/move", post(subsections::move_subsection))
        .route("/notes", post(lecture_notes::create_note))
        .route("/notes/:id", put(lecture_notes::update_note).delete(lecture_notes::delete_note))
        .route("/notes/move", post(lecture_notes::move_note))
        .route("/users", get(users::list_users))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            admin_guard,
        ));

    Router::new()
        .merge(public_routes)
        .merge(admin_routes)
        .with_state(state)
}
