use crate::{Error, Result};
use std::{env, sync::OnceLock};

// NOTE: We don't want to reload the Config ENV again and again.
// We create a helper that returns a &'static Config.
// NOTE: &'static - means it will live to end of program.
// This allows us to access our ENV variable using:
// config().FRONTEND for example.
pub fn config() -> &'static Config {
    // OnceLock takes type you're going to store (Config)
    // NOTE: 'static' keyword is kinda like const as it's global,
    // but static variables are assigned static lifetimes (I think).
    // By adding it inside this fn, we limit its visibility to only Config
    static INSTANCE: OnceLock<Config> = OnceLock::new();

    // Now let's populate our instance
    INSTANCE.get_or_init(|| {
        // Get our Config or panic early since we don't want
        // our app running without Config properly loaded.
        Config::load_from_env()
            .unwrap_or_else(|ex| panic!("FATAL - WHILE LOADING CONFIG - Cause: {ex:?}"))
    })
}

#[allow(non_snake_case)]
pub struct Config {
    // -- Web
    pub FRONTEND: String,
}

impl Config {
    fn load_from_env() -> Result<Config> {
        Ok(Config {
            // -- Web
            // Ideally don't use unwrap(). Meh:
            // FRONTEND: env::var("SERVICE_FRONTEND").unwrap(),
            // Better:
            FRONTEND: get_env("SERVICE_FRONTEND")?,
        })
    }
}

fn get_env(name: &'static str) -> Result<String> {
    env::var(name).map_err(|_| Error::ConfigMissingEnv(name))
}
