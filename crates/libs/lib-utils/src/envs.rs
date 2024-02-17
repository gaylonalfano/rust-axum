use crate::b64::b64u_decode;
use std::{env, str::FromStr};

pub fn get_env(name: &'static str) -> Result<String> {
    env::var(name).map_err(|_| Error::MissingEnv(name))
}

pub fn get_env_base64url_as_u8s(name: &'static str) -> Result<Vec<u8>> {
    // decode() has its own error, but to use our own custom error, we can use map_err()
    b64u_decode(&get_env(name)?).map_err(|_| Error::WrongFormat(name))
}

// NOTE: Using a general parse<T: FromStr> so we can return multiple
// types i.e. i32, i64, etc.
pub fn get_env_parse<T: FromStr>(name: &'static str) -> Result<T> {
    let val = get_env(name)?;
    // We don't want to pass through the parse() error, so instead we map_err to our own error
    // TODO: Could consider expanding map_err closure to specify the expected type.
    val.parse::<T>().map_err(|_| Error::WrongFormat(name))
}

// region:       -- Error
// NOTE: As this grows, we can move into a separate 'errors' module
// U: Adding Clone so we can return our Result<Ctx, AuthFailCtxNotInRequestExt>
// from inside mw_auth.rs
// U: Adding strum_macros to have variant name as string for errors
// U: Adding Serialize so log_request error can serialize into JSON
// Handy trick when Serializing enum is to specify the tag="type" (Variant name)
// and content="data" (internal data for each variant e.g., { id: u64 })
// U: After adding "derive_more::From" dep, we don't have to manually
pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    MissingEnv(&'static str),
    WrongFormat(&'static str),
}

// region:       -- Error Boilerplate
impl core::fmt::Display for Error {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}

impl std::error::Error for Error {}
// endregion:    -- Error Boilerplate

// endregion:    -- Error
