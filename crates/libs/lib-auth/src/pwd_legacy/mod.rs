mod error;
mod hmac_encrypt_hasher;

pub use self::error::{Error, Result};

use crate::auth_config;
pub use crate::pwd_legacy::hmac_encrypt_hasher::encrypt_into_base64url;
use uuid::Uuid;

// NOTE: When a user enters their password to log in,
// their entered password is salted and hashed, and the
// resulting hash value is compared to the stored hash value.
// If the hash values match, the user is authenticated.

// WARN: Jeremy renamed to ContentToHash
pub struct EncryptContent {
    // NOTE: Using String types since it's easy to convert
    // into array of bytes from String.
    // Q: What is Clear mean?
    // A: I think it's the raw, unencrypted string
    pub content: String, // Clear content.
    pub salt: Uuid,
}

/// Encrypt the password with default scheme (multi-scheme comes later)
/// Format is: #scheme#encrypted_content ---- #01#_encrypted_pwd_b64u_
/// EncryptContent is the Clear content to be encrypted
pub fn encrypt_pwd(enc_content: &EncryptContent) -> Result<String> {
    let key = &auth_config().PWD_KEY;

    let encrypted = encrypt_into_base64url(key, enc_content)?;

    // NOTE: We return the scheme along with the encrypted. This way,
    // when we later have multiple schemes (#01#, #02#, #03#, etc),
    // we can match to all schemes when we validate passwords.
    Ok(format!("#01#{encrypted}"))
}

/// Validate if an EncryptContent matches
pub fn validate_pwd(enc_content: &EncryptContent, pwd_ref: &str) -> Result<()> {
    let pwd = encrypt_pwd(enc_content)?;

    if pwd == pwd_ref {
        Ok(())
    } else {
        Err(Error::NotMatching)
    }
}
