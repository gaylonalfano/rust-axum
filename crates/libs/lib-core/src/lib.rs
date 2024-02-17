// NOTE: !! Our 'core' is independent from 'web'
// It consists of the former Context, Event, Model
// and Store components. Auth, RPC, and Utils will
// now have their own separate crates. Finally, the
// Web will become a separate Web-Server Service,
// that can expand to supporting multiple services.
// REF: https://youtu.be/zUxF0kvydJs?t=485
pub mod config;
pub mod ctx;
pub mod model;

// #[cfg(test)] // Commented during early development.
pub mod _dev_utils;

use config::core_config;
