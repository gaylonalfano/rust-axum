// region: -- Modules

use crate::pwd::{EncryptContent, Error, Result};
use hmac::{Hmac, Mac};
use lib_utils::b64::b64u_encode;
use sha2::Sha512;

// endregion: -- Modules

// NOTE: Normalizing everything into base64_url to make it
// easier/versitile to pass things around. This has nothing
// to do with encryption and security.
pub fn encrypt_into_base64url(key: &[u8], enc_content: &EncryptContent) -> Result<String> {
    let EncryptContent { content, salt } = enc_content;

    // -- Create a HMAC-SHA-512 from key
    let mut hmac_sha512 = Hmac::<Sha512>::new_from_slice(key).map_err(|_| Error::KeyFail)?;

    // -- Add content and salt
    hmac_sha512.update(content.as_bytes());
    hmac_sha512.update(salt.as_bytes());

    // -- Finalize and b64u encode
    let hmac_result = hmac_sha512.finalize();
    let result = b64u_encode(hmac_result.into_bytes());

    Ok(result)
}

// WARN: !! U: After multi-crate upgrade this was removed. I'll add it
// back eventually once I can.
// // region:      -- Tests
// #[cfg(test)]
// mod tests {
//     pub type Result<T> = core::result::Result<T, Error>;
//     pub type Error = Box<dyn std::error::Error>; // For tests.
//
//     use super::*;
//     // use anyhow::Result;
//     use rand::RngCore;
//
//     #[test]
//     fn test_encrypt_into_base64url_ok() -> Result<()> {
//         // -- Setup & Fixtures
//         let mut fx_key = [0u8; 64]; // 512 bits = 64 bytes
//         rand::thread_rng().fill_bytes(&mut fx_key);
//         let fx_enc_content = EncryptContent {
//             content: "hello world".to_string(),
//             salt: "some pepper".to_string(),
//         };
//
//         // TODO: Need to fix fx_key and precompute fx_res
//         let fx_res = encrypt_into_base64url(&fx_key, &fx_enc_content)?;
//
//         // -- Exec
//         let res = encrypt_into_base64url(&fx_key, &fx_enc_content)?;
//         println!("->> {res}");
//
//         // -- Check
//         assert_eq!(res, fx_res);
//
//         Ok(())
//     }
// }
// // endregion:   -- Tests
