use axum::{
    handler::HandlerWithoutStateExt,
    http::StatusCode,
    routing::{any_service, MethodRouter},
};
use tower_http::services::ServeDir;

const FRONTEND: &str = "frontend";

// NOTE: Here we can just return a MethodRouter rather than a full Router
// since ServeDir is a service.
pub fn serve_dir() -> MethodRouter {
    async fn handle_404() -> (StatusCode, &'static str) {
        (StatusCode::NOT_FOUND, "Resource not found")
    }

    any_service(ServeDir::new(FRONTEND).not_found_service(handle_404.into_service()))
}

// -- OLD
// fn routes_static() -> Router {
//     // Serve up static files with local file system and as a fallback
//     // This depends/uses on Tower service ServeDir (tower-http crate)
//     // We want a nested path for static file routing
//     Router::new().nest_service("/", get_service(ServeDir::new("./")))
// }
