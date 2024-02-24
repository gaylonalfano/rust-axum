//! The pwd module is responsible for hashing and validating hashes.
//! It follows a multi-scheme hashing code design, allowing each
//! scheme to provide its own hashing and validation methods.
//!
//! Code Design Points:
//!
//! - Exposes two public async functions `hash_pwd(...)` and `validate_pwd(...)`
//! - `ContentToHash` represents the data to be hashed along with the corresponding salt.
//! - `SchemeStatus` is the result of `validate_pwd` which, upon successful validation, indicates
//!   whether the password needs to be re-hashed to adopt the latest scheme.
//! - Internally, the `pwd` module implements a multi-scheme code design with the `Scheme` trait.
//! - The `Scheme` trait exposes sync functions `hash` and `validate` to be implemented for each scheme.
//! - The two public async functions `hash_pwd(...)` and `validate_pwd(...)` call the scheme using
//!   `spawn_blocking` to ensure that long hashing/validation processes do not hinder the execution of smaller tasks.
//! - Schemes are designed to be agnostic of whether they are in an async or sync context, hence they are async-free.

// region:       -- Modules

// Modules
mod error;
mod scheme;

// Re-exports
pub use self::error::{Error, Result};
pub use scheme::SchemeStatus;

// Imports
use crate::pwd::scheme::{get_scheme, Scheme, DEFAULT_SCHEME};
use lazy_regex::regex_captures;
use std::str::FromStr;
use uuid::Uuid;

// endregion:    -- Modules

// NOTE: When a user enters their password to log in,
// their entered password is salted and hashed, and the
// resulting hash value is compared to the stored hash value.
// If the hash values match, the user is authenticated.
// NOTE: !! If a user's pwd originally used Scheme01 when created,
// we can upgrade the password using Scheme02 the next time
// the user logs in (it's the only time we'll have the CLEAR, non-hashed pwd!).
// This means that we need to check which scheme is used for the user's
// login pwd when logging in.
// REF: https://youtu.be/3E0zK5h9zEs?t=2936
// NOTE: Recall that our api_login_handler() will have the State<ModelManager>,
// Cookies, and Json<LoginPayload>, which will have the user's clear password.
// NOTE: U: Added tokio spawn_blocking for Argon2: https://youtu.be/hBsmOjgdrr0?list=PL7r-PXl6ZPcCLvwpdD2Vj1O4CyoFTiHKd&t=179

// region:       -- Types

/// The clean content to hash, with the salt.
///
/// Notes:
///    - Since content is sensitive information, we do NOT implement default debug for this struct.
///    - The clone is only implement for testing
#[cfg_attr(test, derive(Clone))]
pub struct ContentToHash {
    pub content: String, // Clear content
    pub salt: Uuid,
}

// endregion:    -- Types

// region:       -- Public Functions

/// Hash the password with the default scheme
pub async fn hash_pwd(to_hash: ContentToHash) -> Result<String> {
    tokio::task::spawn_blocking(move || hash_for_scheme(DEFAULT_SCHEME, to_hash))
        .await
        .map_err(|_| Error::FailSpawnBlockForHash)?
}

/// Validate if a ContentToHash matches
pub async fn validate_pwd(to_hash: ContentToHash, pwd_ref: String) -> Result<SchemeStatus> {
    // -- Parse the password to see which scheme it is
    // NOTE: This is where our impl FromStr for PwdParts helps
    let PwdParts {
        scheme_name,
        hashed,
    } = pwd_ref.parse()?;

    // NOTE: !! We don't have access to the database from this crate,
    // so we can only validate (can't update) and send back information
    // so that other modules can do all the database related stuff.
    // NOTE: U: We do this first so we don't have to clone the scheme_name
    let scheme_status = if scheme_name == DEFAULT_SCHEME {
        SchemeStatus::Ok
    } else {
        SchemeStatus::Outdated
    };

    // NOTE: Since validte might take time depending on algo, we use tokio's
    // spawn_blocking to avoid locking up the OS thread.
    tokio::task::spawn_blocking(move || validate_for_scheme(&scheme_name, to_hash, hashed))
        .await
        .map_err(|_| Error::FailSpawnBlockForValidate)??;

    Ok(scheme_status)
}

// endregion:    -- Public Functions

// region:       -- Private Types, Functions

fn hash_for_scheme(scheme_name: &str, to_hash: ContentToHash) -> Result<String> {
    // -- Get the scheme
    // NOTE: Box<dyn Scheme> will deref into a Scheme Trait Object,
    // so we'll have Scheme Trait functions.
    // NOTE: We wrap the scheme::Error inside the pwd::Error::Scheme(scheme::Error)
    // with the help of derive_more #[from], which allows us to convert from the
    // scheme::Error (that'd we get from scheme::get_scheme()) to pwd::Error easily.
    let pwd_hashed = get_scheme(scheme_name)?.hash(&to_hash)?;

    Ok(format!("#{scheme_name}#{pwd_hashed}"))
}

fn validate_for_scheme(scheme_name: &str, to_hash: ContentToHash, pwd_ref: String) -> Result<()> {
    get_scheme(scheme_name)?.validate(&to_hash, &pwd_ref)?;

    Ok(())
}

/// Parse the pwd to get the scheme and the hashed part
struct PwdParts {
    /// The scheme only (e.g., "01")
    scheme_name: String,

    /// The hashed password
    hashed: String,
}

impl FromStr for PwdParts {
    type Err = Error;

    // NOTE: The full return type is: std::prelude::v1::Result<Self, Self::Err>
    // but can be simplified bc Result is a type alias of our Error.
    // i.e. (I think...), type Result<T> = core::result::Result<T, Error>
    fn from_str(pwd_with_scheme: &str) -> Result<Self> {
        // Starting out we had 'let dd = regex_captures(...) to see types
        regex_captures!(r#"^#(\w+)#(.*)"#, pwd_with_scheme)
            .map(|(_, scheme, hashed)| Self {
                scheme_name: scheme.to_string(),
                hashed: hashed.to_string(),
            })
            .ok_or(Error::PwdWithSchemeFailedParse)
    }
}

// endregion:    -- Private Types, Functions

// region:       -- Tests

mod tests {
    pub type Result<T> = core::result::Result<T, Error>;
    pub type Error = Box<dyn std::error::Error>; // For tests

    use super::*;

    #[tokio::test]
    async fn test_multi_scheme_ok() -> Result<()> {
        // -- Setup & Fixtures
        // Q: Where does this string come from?
        let fx_salt = Uuid::parse_str("f05e8961-d6ad-4086-9e78-a6de065e5453")?;
        let fx_to_hash = ContentToHash {
            content: "hello world".to_string(),
            salt: fx_salt,
        };

        // -- Exec
        // NOTE: !! U: Auto-upgrade a user's pwd to Scheme02 if originally Scheme01.
        // ONLY our pwd/mod.rs has access to hash_for_scheme(), but since this local
        // 'tests' module is a child of our pwd module, children can see what parents
        // have (i.e., the private function hash_for_scheme()), so it's accessible here.
        // NOTE: U: We enabled Clone for tests only via #[cfg_attr(test, Clone)] for
        // our ContentToHash struct.
        let pwd_hashed = hash_for_scheme("01", fx_to_hash.clone())?;
        // println!("->> pwd_hashed: {pwd_hashed}");
        let pwd_validate = validate_pwd(fx_to_hash.clone(), pwd_hashed).await?;
        // println!("->>   validate: {pwd_validate:?}");

        // -- Check
        // NOTE: !! U: We want to check that we get SchemeStatus::Outdated,
        // since we now are using Scheme02 as default, and we're testing
        // that we will auto-upgrade from Scheme01 to Scheme02.
        // Recall that pwd_validate() -> SchemeStatus
        assert!(
            matches!(pwd_validate, SchemeStatus::Outdated),
            "status should be SchemeStatus::Outdated"
        );

        Ok(())
    }
}
// endregion:    -- Tests
