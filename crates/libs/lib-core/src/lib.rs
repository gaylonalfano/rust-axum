// NOTE: !! Our 'core' is independent from 'web'
// REF: https://youtu.be/zUxF0kvydJs?t=485
pub mod config;
pub mod ctx;
pub mod model;

// #[cfg(test)] // Commented during early development.
pub mod _dev_utils;

use config::core_config;
