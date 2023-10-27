#![allow(unused)] // For beginners

// Re-export our new custom Error and Result from error.rs
// We now have a crate Error and crate Result we can import
// into other modules.
pub use self::error::{Error, Result};

use std::net::SocketAddr;

use axum::{
    extract::{Path, Query},
    http::{Method, Uri},
    middleware,
    response::{Html, IntoResponse, Response},
    routing::{get, get_service},
    Json, Router,
};
use serde::Deserialize;
use serde_json::json;
use tower_cookies::CookieManagerLayer;
use tower_http::services::ServeDir;

// My custom modules following Anchor style conventions
// use instructions::*;
// pub mod instructions;
use ctx::*; // Custom Extractor
use error::*;
use log::*;
use model::*;
use uuid::Uuid;
use web::*;

pub mod ctx;
pub mod error;
pub mod log;
pub mod model;
pub mod web;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize ModelManager
    let mm = ModelManager::new().await?;

    // NOTE: You could create a separate struct for mw, but the from_fn() is very
    // powerful
    // REF: https://youtu.be/XZtlD_m59sM?t=2619
    let routes_api = web::routes_tickets::routes(mm.clone())
        // Q: Why use route_layer() vs. layer()?
        // A: route_layer only applies to this router, so routes_hello & routes_login
        // won't be affected.
        .route_layer(middleware::from_fn(web::mw_auth::mw_require_auth));

    // NOTE: Basically every Axum route handler gets turned into a
    // Tower::Service trait, which is roughly equivalent to sth that
    // implements:
    // async fn (Request) -> Result<Response, E> for some <Request, E>
    // // REF: https://youtu.be/Wnb_n5YktO8?t=1850
    // // REF: https://tokio.rs/blog/2021-05-14-inventing-the-service-trait
    let routes_all = Router::new()
        .merge(routes_hello())
        .merge(web::routes_login::routes())
        // NOTE: By nesting (merging), we are basically attaching a subrouter
        .nest("/api", routes_api)
        .layer(middleware::map_response(main_response_mapper))
        // NOTE: Making our Ctx extractor accessible to all routes
        .layer(middleware::from_fn_with_state(
            mm.clone(),
            web::mw_auth::mw_ctx_resolver,
        ))
        .layer(CookieManagerLayer::new())
        .fallback_service(routes_static());

    // region:  --- Start Server
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    println!("->> LISTENING on {addr}\n");
    axum::Server::bind(&addr)
        .serve(routes_all.into_make_service())
        .await
        .unwrap();
    // region: -- end Start Server

    Ok(())
}

// Adding first layer (middleware)
// REF: Interesting relevant Axum details by Jon Gjengset: https://youtu.be/Wnb_n5YktO8?t=2273
// This is where the "magic" happens:
// REF: https://youtu.be/XZtlD_m59sM?t=4154
// U: Adding our log_request() helper for logging requests per line
// Thanks to Axum's Extractors, we can get all the needed info.
async fn main_response_mapper(
    ctx: Option<Ctx>,
    uri: Uri,
    req_method: Method,
    res: Response,
) -> Response {
    println!("->> {:<12} - main_response_mapper", "RES_MAPPER");
    // Create a uuid to match our server errors to client errors
    let uuid = Uuid::new_v4();

    // - Get the eventual response error
    let service_error = res.extensions().get::<Error>();
    let client_status_error = service_error.map(|se| se.client_status_and_error());

    // - If client error, build a new response
    // Using as_ref() bc we're going to reuse this for server request login
    let error_response = client_status_error
        .as_ref()
        .map(|(status_code, client_error)| {
            let client_error_body = json!({
                "error": {
                    "type": client_error.as_ref(), // Thanks to strum_macros
                    "req_uuid": uuid.to_string(),
                }
            });

            println!("   ->> client error body: {client_error_body}");

            // Build the new reponse from the client_error_body
            // NOTE:Recall we expanded into_response() to be a Axum Response
            // placeholder that takes the actual server error and
            // inserts it into the Response via extensions_mut()
            (*status_code, Json(client_error_body)).into_response()
        });

    // -- Build and log the server log line
    // NOTE: Server Reequests Log Line vs. Tracing
    // Tracing is adding warnings, info, debug, etc. inside your code so you can debug
    // Requests log line is one log line per request with error and other info.
    // You then can push to console.log() locally, and after deploying to the cloud
    // you can then use tools like CloudWatch and query with cloud-native tools.
    // NOTE: Option.unzip() gives us the Option<ClientError>
    let client_error = client_status_error.unzip().1;
    log_request(uuid, req_method, uri, ctx, service_error, client_error).await;

    println!();
    // NOTE:If we remove our quick_dev req_login(), we'll see the error uuids
    // match in the logs for both client and server errors. Neat!
    error_response.unwrap_or(res)
}

fn routes_static() -> Router {
    // Serve up static files with local file system and as a fallback
    // This depends/uses on Tower service ServeDir (tower-http crate)
    // We want a nested path for static file routing
    Router::new().nest_service("/", get_service(ServeDir::new("./")))
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
    println!("->> {:<12} - handler_hello - {params:?}", "HANDLER");

    let name = params.name.as_deref().unwrap_or("World");

    Html(format!("Hello <strong>{name}!</strong>"))
}

// e.g., `/hello2/Mario` -- as a path
async fn handler_hello2(Path(name): Path<String>) -> impl IntoResponse {
    println!("->> {:<12} - handler_hello2 - {name:?}", "HANDLER");

    Html(format!("Hello2 <strong>{name}!</strong>"))
}
