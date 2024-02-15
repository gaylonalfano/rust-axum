use lib_utils::envs::get_env;
use std::sync::OnceLock;

// NOTE: We don't want to reload the CoreConfig ENV again and again.
// We create a helper that returns a &'static Config.
// NOTE: &'static - means it will live to end of program.
// This allows us to access our ENV variable using:
// core_config().FRONTEND for example.
pub fn core_config() -> &'static CoreConfig {
    // OnceLock takes type you're going to store (Config)
    // NOTE: 'static' keyword is kinda like const as it's global,
    // but static variables are assigned static lifetimes (I think).
    // By adding it inside this fn, we limit its visibility to only Config
    static INSTANCE: OnceLock<CoreConfig> = OnceLock::new();

    // Now let's populate our instance
    INSTANCE.get_or_init(|| {
        // Get our CoreConfig or panic early since we don't want
        // our app running without Config properly loaded.
        CoreConfig::load_from_env()
            .unwrap_or_else(|ex| panic!("FATAL - WHILE LOADING CONFIG - Cause: {ex:?}"))
    })
}

#[allow(non_snake_case)]
pub struct CoreConfig {
    // -- Db
    pub DB_URL: String,

    // -- Web
    pub WEB_FOLDER: String,
}

impl Config {
    fn load_from_env() -> lib_utils::envs::Result<CoreConfig> {
        Ok(CoreConfig {
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
