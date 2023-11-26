// pub mod create_ledger;
// pub use create_ledger::*;

// Create sub-module:
pub mod error;
pub mod mw_auth;
pub mod mw_res_map;
pub mod routes_login;
pub mod routes_static;

pub use error::*;
pub use mw_auth::*;
pub use mw_res_map::*;
pub use routes_login::*;
pub use routes_static::*;
use tower_cookies::{Cookie, Cookies};

use crate::crypt::token::generate_web_token;

pub const AUTH_TOKEN: &str = "auth-token";

// U: After adding crypt::token
fn set_token_cookie(cookies: &Cookies, user: &str, salt: &str) -> Result<()> {
    // NOTE: generate_web_token returns a crypt::error::Error, but we
    // want a web::error::Error instead, so need to add Crypt(crypt::Error)
    // variant and a new impl From<crypt::Error> for Error
    let token = generate_web_token(user, salt)?;

    let mut cookie = Cookie::new(AUTH_TOKEN, token.to_string());
    cookie.set_http_only(true);
    // NOTE: !! - Must set cookie path to root "/" because it will default
    // to path of the request (ie. 'api/login')
    cookie.set_path("/");

    cookies.add(cookie);

    Ok(())
}
