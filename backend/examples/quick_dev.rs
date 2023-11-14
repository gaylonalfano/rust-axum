#![allow(unused)] // For beginning only
                  // httpc-test crate is a convenient hot-reloading
                  // http client testing and printing results that
                  // uses reqwest and cookie store
                  // NOTE: We watch the src/ dir on backend, and tests/ (or examples/) dir on frontend
                  // BACKEND: cargo watch -q -c -w src/ -x run
                  // FRONTEND: cargo watch -q -c -w examples/ -x "run --example quick_dev"
                  // NOTE: -q quiet, -c clear, -w watch, -x execute

use anyhow::Result;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<()> {
    let http_client = httpc_test::new_client("http://localhost:8080")?;

    // FIXME:
    // http_client.do_get("index.html").await?.print().await?;

    // Setting up our watcher for hot reloading and logs, etc.
    // NOTE: In a separate terminal we run:
    // $> cargo watch -q -c -w src/ -x run
    // NOTE: -q quiet, -c clear, -w watch, -x execute
    http_client
        .do_get("/hello?name=Mario")
        .await?
        .print()
        .await?;

    http_client.do_get("/hello2/Luigi").await?.print().await?;

    // U: We want to add authentication before we're able to
    // perform any CRUD methods. To do this, we're adding middleware (mw_auth.rs)
    let req_login = http_client.do_post(
        "/api/login",
        json!({
            "username": "demo1",
            "pwd": "welcome"
        }),
    );
    // NOTE: Comment out this to test out some error logging
    req_login.await?.print().await?;

    Ok(())
}
