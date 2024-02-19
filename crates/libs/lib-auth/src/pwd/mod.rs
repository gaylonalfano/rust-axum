// region:       -- Modules

// Modules
mod error;
mod scheme;

// Re-exports
pub use self::error::{Error, Result};

// Imports
use uuid::Uuid;

// endregion:    -- Modules

// NOTE: When a user enters their password to log in,
// their entered password is salted and hashed, and the
// resulting hash value is compared to the stored hash value.
// If the hash values match, the user is authenticated.

// region:       -- Types

pub struct ContentToHash {
    pub content: String, // Clear content
    pub salt: Uuid,
}

// endregion:    -- Types

// region:       -- Public Functions

/// Hash the password with the default scheme
pub fn hash_pwd(to_hash: &ContentToHash) -> Result<String> {
    // FIXME:
    Ok("FIXME hash_pwd".to_string())
}

/// Validate if a ContentToHash matches
pub fn validate_pwd(to_hash: &ContentToHash, pwd_ref: &str) -> Result<String> {
    // FIXME:
    Ok("FIXME validate_pwd".to_string())
}

// endregion:    -- Public Functions

// region:       -- Private Types, Functions

// endregion:    -- Private Types, Functions

// region:       -- Tests

mod tests {
    pub type Result<T> = core::result::Result<T, Error>;
    pub type Error = Box<dyn std::error::Error>; // For tests

    use super::*;

    #[test]
    fn test_multi_scheme_ok() -> Result<()> {
        // -- Setup & Fixtures
        let fx_salt = Uuid::parse_str("f05e8961-d6ad-4086-9e78-a6de065e5453")?;
        let fx_to_hash = ContentToHash {
            content: "hello world".to_string(),
            salt: fx_salt,
        };

        // -- Exec
        let pwd_hashed = hash_pwd(&fx_to_hash)?;
        println!("->> pwd_hashed: {pwd_hashed}");
        let pwd_validate = validate_pwd(&fx_to_hash, &pwd_hashed)?;
        println!("->>   validate: {pwd_validate:?}");

        Ok(())
    }
}
// endregion:    -- Tests
