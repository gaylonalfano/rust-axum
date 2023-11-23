use crate::{Error, Result};
use std::{env, str::FromStr, sync::OnceLock};

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
    // -- Crypt
    pub PWD_KEY: Vec<u8>,

    pub TOKEN_KEY: Vec<u8>,
    pub TOKEN_DURATION_SEC: f64,

    // -- Db
    pub DB_URL: String,

    // -- Web
    pub WEB_FOLDER: String,
}

impl Config {
    fn load_from_env() -> Result<Config> {
        Ok(Config {
            // -- Crypt
            PWD_KEY: get_env_base64url_as_u8s("SERVICE_PWD_KEY")?,

            TOKEN_KEY: get_env_base64url_as_u8s("SERVICE_TOKEN_KEY")?,
            TOKEN_DURATION_SEC: get_env_parse("SERVICE_TOKEN_DURATION_SEC")?,

            // -- Db
            DB_URL: get_env("SERVICE_DB_URL")?,

            // -- Web
            // Ideally don't use unwrap().
            // Meh:
            // FRONTEND: env::var("SERVICE_WEB_FOLDER").unwrap(),
            // Better:
            WEB_FOLDER: get_env("SERVICE_WEB_FOLDER")?,
        })
    }
}

fn get_env(name: &'static str) -> Result<String> {
    env::var(name).map_err(|_| Error::ConfigMissingEnv(name))
}

fn get_env_base64url_as_u8s(name: &'static str) -> Result<Vec<u8>> {
    // decode() has its own error, but to use our own custom error, we can use map_err()
    base64_url::decode(&get_env(name)?).map_err(|_| Error::ConfigWrongFormat(name))
}

// NOTE: Using a general parse<T: FromStr> so we can return multiple
// types i.e. i32, i64, etc.
fn get_env_parse<T: FromStr>(name: &'static str) -> Result<T> {
    let val = get_env(name)?;
    // We don't want to pass through the parse() error, so instead we map_err to our own error
    // TODO: Could consider expanding map_err closure to specify the expected type.
    val.parse::<T>().map_err(|_| Error::ConfigWrongFormat(name))
}
