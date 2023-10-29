#![allow(unused)] // For beginners

pub mod config;
pub mod ctx;
pub mod error;
pub mod log;
pub mod model;
pub mod web;

// Re-export our new custom Error and Result from error.rs
// We now have a crate Error and crate Result we can import
// into other modules.
pub use self::error::{Error, Result};
pub use config::*;

use axum::{
    extract::{Path, Query},
    http::{Method, Uri},
    middleware,
    response::{Html, IntoResponse, Response},
    routing::{get, get_service},
    Json, Router,
};
use ctx::*; // Custom Extractor
use error::*;
use log::*;
use model::*;
use serde::Deserialize;
use serde_json::json;
use std::net::SocketAddr;
use tower_cookies::CookieManagerLayer;
use tower_http::services::ServeDir;
use tracing::{debug, info};
use tracing_subscriber::EnvFilter;
use uuid::Uuid;
use web::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Tracing
    tracing_subscriber::fmt()
        .without_time() // E.g. 2023-10-28T13:01:17.945497Z
        .with_target(false) // For simple tracing
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    // Initialize ModelManager
    let mm = ModelManager::new().await?;

    // NOTE: You could create a separate struct for mw, but the from_fn() is very
    // powerful
    // REF: https://youtu.be/XZtlD_m59sM?t=2619
    // NOTE: Basically every Axum route handler gets turned into a
    // Tower::Service trait, which is roughly equivalent to sth that
    // implements:
    // async fn (Request) -> Result<Response, E> for some <Request, E>
    // // REF: https://youtu.be/Wnb_n5YktO8?t=1850
    // // REF: https://tokio.rs/blog/2021-05-14-inventing-the-service-trait
    let routes_all = Router::new()
        .merge(routes_hello())
        .merge(routes_login::routes())
        // NOTE: By nesting (merging), we are basically attaching a subrouter
        // .nest("/api", routes_api)
        .layer(middleware::map_response(mw_response_map))
        // NOTE: Making our Ctx extractor accessible to all routes
        .layer(middleware::from_fn_with_state(mm.clone(), mw_ctx_resolve))
        .layer(CookieManagerLayer::new())
        .fallback_service(routes_static::serve_dir());

    // region:  --- Start Server
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    info!("->> {:<12} - {addr}\n", "LISTENING");
    axum::Server::bind(&addr)
        .serve(routes_all.into_make_service())
        .await
        .unwrap();
    // region: -- end Start Server

    Ok(())
}

// Create a sub-router (like Chi in Go) and merge with main router
fn routes_hello() -> Router {
    Router::new()
        .route("/hello", get(handler_hello))
        .route("/hello2/:name", get(handler_hello2))
}

#[derive(Debug, Deserialize)]
struct HelloParams {
    name: Option<String>,
}

// Using Axum's Query extractor helper that deserializes query strings into some type
// e.g., `/hello?name=Mario` -- as a query string
async fn handler_hello(Query(params): Query<HelloParams>) -> impl IntoResponse {
    debug!(" {:<12} - handler_hello - {params:?}", "HANDLER");

    let name = params.name.as_deref().unwrap_or("World");

    Html(format!("Hello <strong>{name}!</strong>"))
}

// e.g., `/hello2/Mario` -- as a path
async fn handler_hello2(Path(name): Path<String>) -> impl IntoResponse {
    debug!(" {:<12} - handler_hello2 - {name:?}", "HANDLER");

    Html(format!("Hello2 <strong>{name}!</strong>"))
}
