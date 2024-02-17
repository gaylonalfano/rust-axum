// Create sub-module:
mod error;
pub mod mw_auth;
pub mod mw_res_map;
pub mod routes_login;
pub mod routes_rpc;
pub mod routes_static;

pub use self::error::ClientError;
pub use self::error::{Error, Result};
use lib_auth::token::generate_web_token;
use tower_cookies::{Cookie, Cookies};
use uuid::Uuid;

pub const AUTH_TOKEN: &str = "auth-token";

fn set_token_cookie(cookies: &Cookies, user: &str, salt: Uuid) -> Result<()> {
    // NOTE: generate_web_token returns a crypt::error::Error, but we
    // want a web::error::Error instead, so need to add Crypt(crypt::Error)
    // variant and a new impl From<crypt::Error> for Error
    let token = generate_web_token(user, salt)?;

    let mut cookie = Cookie::new(AUTH_TOKEN, token.to_string());
    // NOTE: set_http_only means JS won't be able to access it
    cookie.set_http_only(true);
    // NOTE: !! - Must set cookie path to root "/" because it will default
    // to path of the request (ie. 'api/login')
    cookie.set_path("/");

    cookies.add(cookie);

    Ok(())
}

fn remove_token_cookie(cookies: &Cookies) -> Result<()> {
    let mut cookie = Cookie::from(AUTH_TOKEN);
    // NOTE: !! - Must set cookie path to root "/" because it will default
    // to path of the request (e.g., 'api/login')
    cookie.set_path("/");

    cookies.remove(cookie);

    Ok(())
}
