// region: -- Modules
mod error;
pub mod pwd;
pub mod token;

pub use self::error::{Error, Result};

use crate::utils::b64u_encode;
use hmac::{Hmac, Mac};
use sha2::Sha512;
// endregion: -- Modules

// NOTE: When a user enters their password to log in,
// their entered password is salted and hashed, and the
// resulting hash value is compared to the stored hash value.
// If the hash values match, the user is authenticated.

pub struct EncryptContent {
    // NOTE: Using String types since it's easy to convert
    // into array of bytes from String.
    // Q: What is Clear mean?
    // A: I think it's the raw, unencrypted string
    pub content: String, // Clear content.
    pub salt: String,    // Clear salt.
}

// NOTE: Normalizing everything into base64_url to make it
// easier/versitile to pass things around. This has nothing
// to do with encryption and security.
pub fn encrypt_into_base64url(key: &[u8], enc_content: &EncryptContent) -> Result<String> {
    let EncryptContent { content, salt } = enc_content;

    // -- Create a HMAC-SHA-512 from key
    let mut hmac_sha512 = Hmac::<Sha512>::new_from_slice(key).map_err(|_| Error::KeyFailHmac)?;

    // -- Add content and salt
    hmac_sha512.update(content.as_bytes());
    hmac_sha512.update(salt.as_bytes());

    // -- Finalize and b64u encode
    let hmac_result = hmac_sha512.finalize();
    let result = b64u_encode(hmac_result.into_bytes());

    Ok(result)
}

// region:      -- Tests
#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use rand::RngCore;

    #[test]
    fn test_encrypt_into_base64url_ok() -> Result<()> {
        // -- Setup & Fixtures
        let mut fx_key = [0u8; 64]; // 512 bits = 64 bytes
        rand::thread_rng().fill_bytes(&mut fx_key);
        let fx_enc_content =
            EncryptContent {
                content: "hello world".to_string(),
                salt: "some pepper".to_string(),
            };

        // TODO: Need to fix fx_key and precompute fx_res
        let fx_res = encrypt_into_base64url(&fx_key, &fx_enc_content)?;

        // -- Exec
        let res = encrypt_into_base64url(&fx_key, &fx_enc_content)?;
        println!("->> {res}");

        // -- Check
        assert_eq!(res, fx_res);

        Ok(())
    }
}
// endregion:   -- Tests
