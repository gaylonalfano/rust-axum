use lib_utils::envs::{get_env_base64url_as_u8s, get_env_parse};
use std::sync::OnceLock;

// NOTE: We don't want to reload the AuthConfig ENV again and again.
// We create a helper that returns a &'static Config.
// NOTE: &'static - means it will live to end of program.
// This allows us to access our ENV variable using:
// auth_config().FRONTEND for example.
pub fn auth_config() -> &'static AuthConfig {
    // OnceLock takes type you're going to store (Config)
    // NOTE: 'static' keyword is kinda like const as it's global,
    // but static variables are assigned static lifetimes (I think).
    // By adding it inside this fn, we limit its visibility to only Config
    static INSTANCE: OnceLock<AuthConfig> = OnceLock::new();

    // Now let's populate our instance
    INSTANCE.get_or_init(|| {
        // Get our AuthConfig or panic early since we don't want
        // our app running without Config properly loaded.
        AuthConfig::load_from_env()
            .unwrap_or_else(|ex| panic!("FATAL - WHILE LOADING CONFIG - Cause: {ex:?}"))
    })
}

#[allow(non_snake_case)]
pub struct AuthConfig {
    // -- Crypt
    pub PWD_KEY: Vec<u8>,

    pub TOKEN_KEY: Vec<u8>,
    pub TOKEN_DURATION_SEC: f64,
}

impl AuthConfig {
    fn load_from_env() -> lib_utils::envs::Result<AuthConfig> {
        Ok(AuthConfig {
            // -- Crypt
            PWD_KEY: get_env_base64url_as_u8s("SERVICE_PWD_KEY")?,

            TOKEN_KEY: get_env_base64url_as_u8s("SERVICE_TOKEN_KEY")?,
            TOKEN_DURATION_SEC: get_env_parse("SERVICE_TOKEN_DURATION_SEC")?,
        })
    }
}
