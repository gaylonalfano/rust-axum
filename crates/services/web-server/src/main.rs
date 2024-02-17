// #![allow(unused)] // For beginners

mod config;
mod error;
mod log;
mod web;

// Re-export our new custom Error and Result from error.rs
// We now have a crate Error and crate Result we can import
// into other modules.
pub use self::error::{Error, Result};
pub use config::web_config;

use crate::web::{
    mw_auth::{mw_ctx_require, mw_ctx_resolve},
    mw_res_map::mw_response_map,
    routes_login, routes_rpc, routes_static,
};
use axum::{middleware, Router};
use lib_core::_dev_utils;
use lib_core::model::ModelManager;
use tokio::net::TcpListener;
use tower_cookies::CookieManagerLayer;
use tracing::info;
use tracing_subscriber::EnvFilter;

// endregion:    -- Modules
#[tokio::main]
async fn main() -> Result<()> {
    // -- Enable RUST_BACKTRACE
    // env::set_var("RUST_BACKTRACE", "1");

    // -- Tracing
    tracing_subscriber::fmt()
        .without_time() // E.g. 2023-10-28T13:01:17.945497Z
        .with_target(false) // For simple tracing
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    // -- FOR DEV ONLY
    // NOTE: We don't use '?' shorthand so it will fail if it
    // doesn't initialize correctly.
    _dev_utils::init_dev().await;

    // -- Initialize ModelManager
    let mm = ModelManager::new().await?;

    // -- Define Routes
    let routes_rpc =
        routes_rpc::routes(mm.clone()).route_layer(middleware::from_fn(mw_ctx_require));

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
        .merge(routes_login::routes(mm.clone()))
        // NOTE: By nesting (merging), we are basically attaching a subrouter
        .nest("/api", routes_rpc)
        .layer(middleware::map_response(mw_response_map))
        // NOTE: Making our Ctx extractor accessible to all routes
        .layer(middleware::from_fn_with_state(mm.clone(), mw_ctx_resolve))
        .layer(CookieManagerLayer::new())
        .fallback_service(routes_static::serve_dir());

    // region:  --- Start Server
    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
    info!("{:<12} - {:?}\n", "LISTENING", listener.local_addr());
    axum::serve(listener, routes_all.into_make_service())
        .await
        .unwrap();
    // region: -- end Start Server

    Ok(())
}
