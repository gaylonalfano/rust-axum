#![allow(unused)] // For beginning only
                  // httpc-test crate is a convenient hot-reloading
                  // http client testing and printing results that
                  // uses reqwest and cookie store
                  // NOTE: We watch the src/ dir on backend, and tests/ (or examples/) dir on frontend
                  // BACKEND: cargo watch -q -c -w src/ -x run
                  // FRONTEND: cargo watch -q -c -w tests/ -x "test -q quick_dev -- --nocapture"
                  // NOTE: -q quiet, -c clear, -w watch, -x execute

use anyhow::Result;
use serde_json::json;

#[tokio::test]
async fn quick_dev() -> Result<()> {
    let http_client = httpc_test::new_client("http://localhost:8080")?;

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
    req_login.await?.print().await?;

    // U: I've added a ModelManager { mc: ModelController } struct at this point
    let req_create_ticket_a = http_client.do_post(
        "/api/tickets",
        json!({
        "title": "Ticket AAA"
        }),
    );
    req_create_ticket_a.await?.print().await?;

    let req_create_ticket_b = http_client.do_post(
        "/api/tickets",
        json!({
        "title": "Ticket BBB"
        }),
    );
    req_create_ticket_b.await?.print().await?;

    http_client
        .do_delete("/api/tickets/1")
        .await?
        .print()
        .await?;

    http_client.do_get("/api/tickets").await?.print().await?;

    // For fallback usually point to a web folder instead of main.rs
    // http_client.do_get("/src/main.rs").await?.print().await?;

    Ok(())
}
