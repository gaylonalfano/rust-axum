use lib_utils::envs::get_env;
use std::sync::OnceLock;

// NOTE: We don't want to reload the Config ENV again and again.
// We create a helper that returns a &'static Config.
// NOTE: &'static - means it will live to end of program.
// This allows us to access our ENV variable using:
// config().FRONTEND for example.
// NOTE: U: Multi-crate workspace changing to WebConfig
pub fn web_config() -> &'static WebConfig {
    // OnceLock takes type you're going to store (Config)
    // NOTE: 'static' keyword is kinda like const as it's global,
    // but static variables are assigned static lifetimes (I think).
    // By adding it inside this fn, we limit its visibility to only Config
    static INSTANCE: OnceLock<WebConfig> = OnceLock::new();

    // Now let's populate our instance
    INSTANCE.get_or_init(|| {
        // Get our Config or panic early since we don't want
        // our app running without Config properly loaded.
        WebConfig::load_from_env()
            .unwrap_or_else(|ex| panic!("FATAL - WHILE LOADING CONFIG - Cause: {ex:?}"))
    })
}

#[allow(non_snake_case)]
pub struct WebConfig {
    // -- Web
    pub WEB_FOLDER: String,
}

impl WebConfig {
    fn load_from_env() -> lib_utils::envs::Result<WebConfig> {
        Ok(WebConfig {
            // -- Web
            // Ideally don't use unwrap().
            // Meh:
            // FRONTEND: env::var("SERVICE_WEB_FOLDER").unwrap(),
            // Better:
            WEB_FOLDER: get_env("SERVICE_WEB_FOLDER")?,
        })
    }
}
