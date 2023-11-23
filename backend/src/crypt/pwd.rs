// use super::{Error, Result};
use crate::config;
use crate::crypt::{encrypt_into_base64url, EncryptContent, Error, Result};

/// Encrypt the password with default scheme (multi-scheme comes later)
/// Format is: #scheme#encrypted_content ---- #01#_encrypted_pwd_b64u_
/// EncryptContent is the Clear content to be encrypted
pub fn encrypt_pwd(enc_content: &EncryptContent) -> Result<String> {
    let key = &config().PWD_KEY;

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
        Err(Error::PwdNotMatching)
    }
}
