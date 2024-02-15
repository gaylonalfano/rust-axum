// region:       -- Modules
mod error;

pub use self::error::{Error, Result};

use crate::config::auth_config;
use lib_utils::b64::{b64u_decode_to_string, b64u_encode};
use lib_utils::time::{now_utc, now_utc_plus_sec_str, parse_utc};
use sha2::Sha512;
use std::fmt::Display;
use std::str::FromStr;
use uuid::Uuid;

// endregion:    -- Modules

// FIXME: Complicated topic. Watch for the general Web Token strategy:
// REF: https://youtu.be/-9K7zNgsbP0
// FIXME: Q: Is there a resource that Jeremy can provide that details
// what's needed for web token authentication?

// region:       -- Token Type

/// String format: `identifier_b64u.expiration_b64u.signature_b64u`
// NOTE: Signature is already b64u because we just want to match it
// REF: https://youtu.be/3cA_mk4vdWY?t=9346
#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct Token {
    pub ident: String,     // Identifier (e.g., username).
    pub exp: String,       // Expiration date in Rfc3339.
    pub sign_b64u: String, // Signature, base64url encoded.
}

impl FromStr for Token {
    // Using custom crypt::error Error with Token variants
    type Err = Error;

    fn from_str(token_str: &str) -> std::result::Result<Self, Self::Err> {
        let splits: Vec<&str> = token_str.split('.').collect();
        if splits.len() != 3 {
            return Err(Error::InvalidFormat);
        }
        let (ident_b64u, exp_b64u, sign_b64u) = (splits[0], splits[1], splits[2]);

        Ok(Self {
            ident: b64u_decode_to_string(ident_b64u).map_err(|_| Error::CannotDecodeIdent)?,
            exp: b64u_decode_to_string(exp_b64u).map_err(|_| Error::CannotDecodeExp)?,
            sign_b64u: sign_b64u.to_string(),
        })
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}.{}.{}",
            b64u_encode(&self.ident),
            b64u_encode(&self.exp),
            self.sign_b64u
        )
    }
}

// endregion:    -- Token Type

// region:       -- Web Token Gen & Validation
// NOTE: Here we know which keys to take

pub fn generate_web_token(user: &str, salt: &str) -> Result<Token> {
    let config = &auth_config();
    _generate_token(user, config.TOKEN_DURATION_SEC, salt, &config.TOKEN_KEY)
}

pub fn validate_web_token(origin_token: &Token, salt: &str) -> Result<()> {
    let config = &auth_config();
    _validate_token_sign_and_exp(origin_token, salt, &config.TOKEN_KEY)?;

    Ok(())
}

// endregion:    -- Web Token Gen & Validation

// region:       -- (private) Token Gen & Validation
// NOTE: Here we don't know the specifics of the web token

// NOTE: TIP: When private and public fn names match, best practice
// is to use `_fn_name` for the private version.
fn _generate_token(ident: &str, duration_sec: f64, salt: &str, key: &[u8]) -> Result<Token> {
    // -- Compute the first two components
    let ident = ident.to_string();
    let exp = now_utc_plus_sec_str(duration_sec);

    // -- Sign the first two components
    let sign_b64u = _token_sign_into_b64u(&ident, &exp, salt, key)?;

    Ok(Token {
        ident,
        exp,
        sign_b64u,
    })
}

// Return Err if validate fail
fn _validate_token_sign_and_exp(origin_token: &Token, salt: &str, key: &[u8]) -> Result<()> {
    // -- Validate signature
    let new_sign_b64u = _token_sign_into_b64u(&origin_token.ident, &origin_token.exp, salt, key)?;

    if new_sign_b64u != origin_token.sign_b64u {
        return Err(Error::TokenSignatureNotMatching);
    }

    // -- Validate expiration
    // Need to map to a Token Error
    let origin_exp = parse_utc(&origin_token.exp).map_err(|_| Error::TokenExpNotIso)?;
    let now = now_utc();

    // Ensure that it's not expired
    if origin_exp < now {
        return Err(Error::TokenExpired);
    }

    Ok(())
}

/// Create token signature from token parts and salt
fn _token_sign_into_b64u(ident: &str, exp: &str, salt: &str, key: &[u8]) -> Result<String> {
    // -- Create the content to be signed
    let content = format!("{}.{}", b64u_encode(ident), b64u_encode(exp));
    let signature = encrypt_into_base64url(
        key,
        &EncryptContent {
            content,
            salt: salt.to_string(),
        },
    )?;

    Ok(signature)
}
// endregion:    -- (private) Token Gen & Validation

// region:       -- Tests
#[cfg(test)]
mod tests {
    use std::{fmt::format, thread, time::Duration};

    use super::*;
    use anyhow::Result;

    // NOTE: TIP: For testing `impl Display for Token`, go ahead
    // and create a test at the same time and print what
    // Display is implementing to confirm.
    // NOTE: TIP: We first do simple prints and writes,
    // add the asserts, then finally remove prints to iterate.
    #[test]
    fn test_token_display_ok() -> Result<()> {
        // -- Fixtures
        let fx_token_str = "ZngtaWRlbnQtMDE.MjAyMy0xMS0yNVQxMTozMDowMFo.some-sign-b64u-encoded";
        let fx_token = Token {
            ident: "fx-ident-01".to_string(),
            exp: "2023-11-25T11:30:00Z".to_string(),
            sign_b64u: "some-sign-b64u-encoded".to_string(),
        };

        // -- Exec & Check
        // This will print whatever Display is implementing
        // println!("->> {fx_token}");
        assert_eq!(fx_token.to_string(), fx_token_str);

        // -- Clean
        Ok(())
    }

    #[test]
    fn test_token_from_str_ok() -> Result<()> {
        // -- Fixtures
        let fx_token_str = "ZngtaWRlbnQtMDE.MjAyMy0xMS0yNVQxMTozMDowMFo.some-sign-b64u-encoded";
        let fx_token = Token {
            ident: "fx-ident-01".to_string(),
            exp: "2023-11-25T11:30:00Z".to_string(),
            sign_b64u: "some-sign-b64u-encoded".to_string(),
        };

        // -- Exec
        let token: Token = fx_token_str.parse()?;

        // -- Check
        // NOTE: You could use PartialEq macro trait on Token
        // to compare two structs, but another approach is to
        // just use debug formatting to compare.
        assert_eq!(format!("{token:?}"), format!("{fx_token:?}"));

        // -- Clean
        Ok(())
    }

    #[test]
    fn test_validate_web_token_ok() -> Result<()> {
        // -- Setup & Fixtures
        let fx_user = "user_one";
        let fx_salt = "pepper";
        let fx_duration_sec = 0.02; // 20ms
                                    // NOTE: Could consider creating a full Token in config instead
        let token_key = &auth_config().TOKEN_KEY;
        let fx_token = _generate_token(fx_user, fx_duration_sec, fx_salt, token_key)?;

        // -- Exec
        thread::sleep(Duration::from_millis(10));
        let res = validate_web_token(&fx_token, fx_salt);

        // -- Check
        res?;

        Ok(())
    }

    #[test]
    fn test_validate_web_token_err_expired() -> Result<()> {
        // -- Setup & Fixtures
        let fx_user = "user_one";
        let fx_salt = "pepper";
        let fx_duration_sec = 0.01; // 10ms

        let token_key = &auth_config().TOKEN_KEY;
        let fx_token = _generate_token(fx_user, fx_duration_sec, fx_salt, token_key)?;

        // -- Exec
        // NOTE: Our fx_token expiration should have passed after sleeping for 20ms
        thread::sleep(Duration::from_millis(20));
        let res = validate_web_token(&fx_token, fx_salt);

        // -- Check
        // Q: How to assert we get the intended Error from our Result?
        // A: Use assert!(matches!(...)) macros!
        assert!(
            matches!(res, Err(Error::TokenExpired)),
            "Should have matched `Err(Error::TokenExpired)` but was `{res:?}`"
        );

        Ok(())
    }
}
// endregion:    -- Tests
