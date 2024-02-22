// region:       -- Modules

// Modules
mod error;
mod scheme_01;

// Re-exports
pub use self::error::{Error, Result};

// Imports
use crate::pwd::ContentToHash;

// endregion:    -- Modules

pub const DEFAULT_SCHEME: &str = "01";

// NOTE: !! Goal is to turn this Scheme trait into a Trait Object (i.e., has a ref of self &self).
// REF: https://youtu.be/3E0zK5h9zEs?t=715
// NOTE: !! This scheme does not know if it's the latest or outdated! It could be HMAC512 or Argon2 scheme.
// but we'll use another function to check whether it's latest or outdated.
pub trait Scheme {
    // NOTE: Taking &self makes this a Trait Object
    fn hash(&self, to_hash: &ContentToHash) -> Result<String>;

    fn validate(&self, to_hash: &ContentToHash, pwd_ref: &str) -> Result<()>;
}

#[derive(Debug)]
pub enum SchemeStatus {
    Ok, // The pwd uses the latest scheme. All good.
    // NOTE: If it's outdated, then our code can rehash it
    Outdated, // The pwd uses an old scheme
}

// NOTE: Box<dyn Scheme> ->> Box of a Trait Object that implements Scheme Trait
// TODO: We start with Dynamic Dispatch (dyn), but later will convert to Static Dispatch
// with Enum Dispatch.
pub fn get_scheme(scheme_name: &str) -> Result<Box<dyn Scheme>> {
    match scheme_name {
        "01" => Ok(Box::new(scheme_01::Scheme01)),
        _ => Err(Error::SchemeNotFound(scheme_name.to_string())),
    }
}
