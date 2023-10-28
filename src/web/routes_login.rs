use crate::web::{self, Error, Result};
use axum::{routing::post, Json, Router};
use serde::Deserialize;
use serde_json::{json, Value};
use tower_cookies::{Cookie, Cookies};

// Common practice is to create a fn that returns the module Router
// and then merge(web::routes_login::routes()) inside main
pub fn routes() -> Router {
    Router::new().route("/api/login", post(api_login_handler))
}

// NOTE: We can return our crate::Result bc Error has impl into_response()
async fn api_login_handler(cookies: Cookies, payload: Json<LoginPayload>) -> Result<Json<Value>> {
    println!("->> {:<12} - api_login_handler", "HANDLER");

    // TODO: Implement real db/auth logic
    if payload.username != "demo1" || payload.pwd != "welcome" {
        return Err(Error::LoginFail);
    }

    // TODO: Implement real auth-token generation/signature
    // Set cookies using Tower's CookieManagerLayer extractor
    // We'll use a format of: "user-{id}.{expire_date}.{signature}"
    cookies.add(Cookie::new(web::AUTH_TOKEN, "user-1.exp.sign"));

    // Create the success body
    let body = Json(json!({
        "result": {
        "success": true
        }
    }));

    Ok(body)
}
// Login  payload sent from client
// Deserialized from JSON to Rust
#[derive(Debug, Deserialize)]
struct LoginPayload {
    username: String,
    pwd: String,
}
