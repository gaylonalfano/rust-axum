use crate::web::{self, remove_token_cookie, Error, Result};
use axum::{extract::State, routing::post, Json, Router};
use lib_auth::pwd_legacy::{self, EncryptContent};
use lib_core::ctx::Ctx;
use lib_core::model::user::{UserBmc, UserForLogin};
use lib_core::model::ModelManager;
use serde::Deserialize;
use serde_json::{json, Value};
use tower_cookies::Cookies;
use tracing::debug;

// Common practice is to create a fn that returns the module Router
// and then merge(web::routes_login::routes()) inside main
// NOTE: U: Adding ModelManager to the routes so we can add AppState
// using with_state() for real db/auth login logic. We then use
// Axum's State extractor in the handlers. From main.rs, we simply
// just pass mm.clone() to the router.
// NOTE: U: Adding new logoff route
pub fn routes(mm: ModelManager) -> Router {
    Router::new()
        .route("/api/login", post(api_login_handler))
        .route("/api/logoff", post(api_logoff_handler))
        .with_state(mm)
}

// region:       -- Login
// NOTE: We can return our crate::Result bc Error has impl into_response()
// NOTE: U: After adding with_state(mm) to the route, we can now use
// Axum's State(mm) extractor to give us access the UserBmc for logging in.
async fn api_login_handler(
    State(mm): State<ModelManager>,
    cookies: Cookies,
    Json(payload): Json<LoginPayload>,
) -> Result<Json<Value>> {
    debug!("{:<12} - api_login_handler", "HANDLER");

    // -- Get payload & System user (Ctx::root_ctx())
    let LoginPayload {
        username,
        pwd: pwd_clear,
    } = payload;
    // Obtain our root Ctx to get the system user for login, since
    // this current user cannot be used (currently trying to log in)
    let root_ctx = Ctx::root_ctx();

    // -- Get the current User
    // NOTE: !! - We don't want to capture/log the username anywhere!
    // Sometimes users accidentally enter their pwd for their username.
    // This makes it harder for us to debug, but it's necessary.
    // NOTE: Our handler returns a web module Result, which will return
    // a web layer Error (.await?) if Err variant. We need to let it convert from a
    // web Error -> model Error. To do this, we need to update our web::error
    // sub module and impl From<model::Error> for Error (web).
    let user: UserForLogin = UserBmc::first_by_username(&root_ctx, &mm, &username)
        .await?
        .ok_or(Error::LoginFailUsernameNotFound)?;
    let user_id = user.id;

    // -- Validate the password
    // NOTE: let-else pattern for adding a guard on password
    let Some(pwd) = user.pwd else {
        return Err(Error::LoginFailUserHasNoPwd { user_id });
    };

    pwd_legacy::validate_pwd(
        &EncryptContent {
            content: pwd_clear.clone(),
            salt: user.pwd_salt,
        },
        &pwd,
    )
    .map_err(|_| Error::LoginFailPwdNotMatching { user_id })?;

    // // -- Fake Login:
    // // TODO: Implement real db/auth logic
    // if payload.username != "demo1" || payload.pwd != "welcome" {
    //     return Err(Error::LoginFail);
    // }

    // -- Set web token cookies using Tower's CookieManagerLayer extractor
    // We'll use a format of: "user-{id}.{expire_date}.{signature}"
    // - OLD:
    // cookies.add(Cookie::new(web::AUTH_TOKEN, "user-1.exp.sign"));
    // - U: With auth-token gen/sign:
    // REF: https://youtu.be/3cA_mk4vdWY?t=10449
    web::set_token_cookie(&cookies, &user.username, user.token_salt)?;

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
// endregion:    -- Login

// region:       -- Logoff
async fn api_logoff_handler(
    cookies: Cookies,
    Json(payload): Json<LogoffPayload>,
) -> Result<Json<Value>> {
    debug!("{:<12} - api_logoff_handler", "HANDLER");
    let should_logoff = payload.logoff;

    if should_logoff {
        remove_token_cookie(&cookies)?;
    }

    // Create the success body
    let body = Json(json!({
        "result": {
        "logged_off": should_logoff
    }
    }));

    Ok(body)
}

// NOTE: It's a silly struct but we want the logoff to be a POST request,
// and we want it in JSON. TL;DR - Content Application/JSON POST are PREFLIGHTED
// by the browser, so there is some cross-site scripting protection.
#[derive(Debug, Deserialize)]
struct LogoffPayload {
    logoff: bool,
}
// endregion:    -- Logoff
