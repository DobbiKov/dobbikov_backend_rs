use loggit::{info, logger, Level};
use pass_hashing::hash_password;
use std::net::SocketAddr;
use axum::http::header;
use axum::http::Method;
use tower_http::cors::{Any, CorsLayer};

pub mod db;
pub mod examples;
pub mod pass_hashing;
pub mod routes;
pub mod services;

#[cfg(test)]
mod tests;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let pool: sqlx::Pool<sqlx::MySql>;
    logger::set_colorized(true);
    logger::set_level_formatting(Level::INFO, "<green>[{level}]<green>: {message}");
    logger::set_file("log.txt");

    match db::establish_connection().await {
        Ok(conn) => pool = conn,
        Err(_) => {
            panic!("an error occured")
        }
    };
    info!("The connection was successfully established, checking tables");
    db::create_tables::create_required_tables(&pool).await;
    info!("The tables were verified and the missing ones were successfully created");

    let addr: SocketAddr = std::env::var("SERVER_ADDR")
        .unwrap_or_else(|_| "127.0.0.1:3000".to_string())
        .parse()
        .expect("SERVER_ADDR must be a valid socket address");

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION]);

    let register_only_for_admin = std::env::var("REGISTER_ONLY_FOR_ADMIN")
        .unwrap_or_else(|_| "false".to_string())
        .eq_ignore_ascii_case("true");

    let app = routes::router(routes::AppState {
        pool,
        register_only_for_admin,
    })
    .layer(cors);
    info!("Starting server on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind server address");
    axum::serve(listener, app)
        .await
        .expect("server error");
}
