use super::{Error, Result, Scheme};
use crate::config::auth_config;
use argon2::{
    password_hash::SaltString, Argon2, Params, PasswordHash, PasswordHasher, PasswordVerifier,
};
use std::sync::OnceLock;

// NOTE: !! Argon2 specifics: https://youtu.be/3E0zK5h9zEs?t=2623
// - When we validate our pwd, we DON'T re-encode it! Argon stores all of the
// configuration (salt, hasher version, algorithm, etc.) on how to hash the
// password DIRECTLY into the string!
// - This means that we first parse to get the PasswordHash, and then when
// we verify password, we don't pass our salt!
// WARN: This differs from scheme_01. If we change the user password's salt,
// the validation won't work anymore on previous password for scheme_01.
// But for scheme_02 (Argon2), if we change in the database the user password's
// salt, it will still work! Again, this is because everything has been stored
// inside the 'pwd_ref', which has all the info needed to do validation.

pub struct Scheme02;

impl Scheme for Scheme02 {
    fn hash(&self, to_hash: &crate::pwd::ContentToHash) -> Result<String> {
        // -- Get the Argon2 Object
        let argon2 = get_argon2();

        // -- Encode our Salt with base 64
        let salt_b64 = SaltString::encode_b64(to_hash.salt.as_bytes()).map_err(|_| Error::Salt)?;

        // -- Hash password
        let pwd = argon2
            .hash_password(to_hash.content.as_bytes(), &salt_b64)
            .map_err(|_| Error::Hash)?
            .to_string();

        Ok(pwd)
    }

    fn validate(&self, to_hash: &crate::pwd::ContentToHash, pwd_ref: &str) -> Result<()> {
        // NOTE: !! Argon2 specifics:
        // - When we validate our pwd, we DON'T re-encode it! Argon stores all of the
        // configuration (salt, hasher version, algorithm, etc.) on how to hash the
        // password DIRECTLY into the string!
        // - This means that we first parse to get the PasswordHash, and then when
        // we verify password, we don't pass our salt!

        // -- Get the Argon2 Object
        let argon2 = get_argon2();

        // -- Parse pwd with Argon2 parser since Argon2 stores salt, etc.
        let parsed_hash_ref = PasswordHash::new(pwd_ref).map_err(|_| Error::Hash)?;

        // -- Verify password and map to our custom Error::PwdValidate
        argon2
            .verify_password(to_hash.content.as_bytes(), &parsed_hash_ref)
            .map_err(|_| Error::PwdValidate)
    }
}

// NOTE: With Argon2, we first need to get an Argon2 Object
fn get_argon2() -> &'static Argon2<'static> {
    static INSTANCE: OnceLock<Argon2<'static>> = OnceLock::new();

    INSTANCE.get_or_init(|| {
        // Just get the key only once
        let key = &auth_config().PWD_KEY;
        // TODO: We want this to fail very early, so may need this at init(), but we
        // don't want to fail it at the firs login.
        Argon2::new_with_secret(
            key,
            argon2::Algorithm::Argon2id,
            argon2::Version::V0x13,
            Params::default(),
        )
        .unwrap()
    })
}

// region:       -- Tests

#[cfg(test)]
mod tests {
    pub type Result<T> = core::result::Result<T, Error>;
    pub type Error = Box<dyn std::error::Error>;

    use super::*;
    use crate::pwd::ContentToHash;
    use uuid::Uuid;

    // cargo test -p lib-auth test_scheme_02_hash_into_b64u_ok
    #[test]
    fn test_scheme_02_hash_into_b64u_ok() -> Result<()> {
        // -- Setup & Fixtures
        let fx_salt = Uuid::parse_str("f05e8961-d6ad-4086-9e78-a6de065e5453")?;
        // let fx_key = &auth_config().PWD_KEY; // 512 bits = 64 bytes
        let fx_to_hash = ContentToHash {
            content: "hello world".to_string(),
            salt: fx_salt,
        };
        // NOTE: We previously ran/generated this value! This comes from the Argon2
        // that we have in the config.
        // Q: Wait, what? How do we get this exactly?
        // A: From Jeremy: This is what got generated with those values (content and salt).
        // So, I did a println, then, took it as the fx_res. It's kind of a chicken and egg,
        // but at least, it will make sure I always get the same result for the same input.
        let fx_res = "$argon2id$v=19$m=19456,t=2,p=1$8F6JYdatQIaeeKbeBl5UUw$fI1fA9uKoMvSN15tpa5Kv4teBrqLmli+/L9zZVthSNo";

        // -- Exec
        let scheme = Scheme02;
        let res = scheme.hash(&fx_to_hash)?;
        // NOTE: It's this 'res' schem hash that is used as the fx_res string above! Chicken/Egg.
        println!("Scheme02.hash(ContentToHash): {:?}", res);
        // "$argon2id$v=19$m=19456,t=2,p=1$8F6JYdatQIaeeKbeBl5UUw$fI1fA9uKoMvSN15tpa5Kv4teBrqLmli+/L9zZVthSNo"

        // -- Check
        assert_eq!(res, fx_res);

        Ok(())
    }
}
// endregion:    -- Tests
