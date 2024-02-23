// region:       -- Modules

// Modules
mod error;
mod scheme_01;
mod scheme_02;

// Re-exports
pub use self::error::{Error, Result};

// Imports
use crate::pwd::ContentToHash;
use enum_dispatch::enum_dispatch;

// endregion:    -- Modules

pub const DEFAULT_SCHEME: &str = "02";

#[derive(Debug)]
pub enum SchemeStatus {
    Ok, // The pwd uses the latest scheme. All good.
    // NOTE: If it's outdated, then our code can rehash it
    Outdated, // The pwd uses an old scheme
}

// NOTE: !! Goal is to turn this Scheme trait into a Trait Object (i.e., has a ref of self &self).
// REF: https://youtu.be/3E0zK5h9zEs?t=715
// NOTE: !! This scheme does not know if it's the latest or outdated! It could be HMAC512 or Argon2 scheme.
// but we'll use another function to check whether it's latest or outdated.
// NOTE: U: If using 'enum_dispatch' crate, gotta add #[enum_dispatch] attribute.
#[enum_dispatch]
pub trait Scheme {
    // NOTE: Taking &self makes this a Trait Object
    fn hash(&self, to_hash: &ContentToHash) -> Result<String>;

    fn validate(&self, to_hash: &ContentToHash, pwd_ref: &str) -> Result<()>;
}

// region:       -- Static Dispatch (#[enum_dispatch] crate)

// NOTE: With enum_dispatch, we first add #[enum_dispatch]
// to our Scheme trait and then #[enum_dispatch(Scheme)] to
// our SchemeDispatcher enum.

#[enum_dispatch(Scheme)]
enum SchemeDispatcher {
    Scheme01(scheme_01::Scheme01),
    Scheme02(scheme_02::Scheme02),
}

// NOTE: We can return a Result of something that impl Scheme (i.e., SchemeDispatcher),
// and this keeps the SchemeDispatcher private and not exposed outside this module.
pub fn get_scheme(scheme_name: &str) -> Result<impl Scheme> {
    match scheme_name {
        "01" => Ok(SchemeDispatcher::Scheme01(scheme_01::Scheme01)),
        "02" => Ok(SchemeDispatcher::Scheme02(scheme_02::Scheme02)),
        _ => Err(Error::SchemeNotFound(scheme_name.to_string())),
    }
}
// endregion:    -- Static Dispatch (#[enum_dispatch] crate)

// // region:       -- Dynamic Dispatch Example (Original)
//
// // NOTE: Box<dyn Scheme> ->> Box of a Trait Object that implements Scheme Trait
// // Box is a pointer that points to a type that implements the Scheme trait.
// // This is Dynamic Dispatch.
// pub fn get_scheme(scheme_name: &str) -> Result<Box<dyn Scheme>> {
//     match scheme_name {
//         "01" => Ok(Box::new(scheme_01::Scheme01)),
//         "02" => Ok(Box::new(scheme_02::Scheme02)),
//         _ => Err(Error::SchemeNotFound(scheme_name.to_string())),
//     }
// }
//
// // endregion:    -- Dynamic Dispatch Example (Original)
//
// // region:       -- Static Dispatch Example (Manual/Vanilla)
//
// // NOTE: Static Dispatch uses an enum and then implements Scheme trait for that
// // enum.
// // NOTE: The crate 'enum_dispatch' basically does this for us via proc macros
// enum SchemeDispatcher {
//     Scheme01(scheme_01::Scheme01),
//     Scheme02(scheme_02::Scheme02),
// }
//
// impl Scheme for SchemeDispatcher {
//     fn hash(&self, to_hash: &ContentToHash) -> Result<String> {
//         match self {
//             SchemeDispatcher::Scheme01(s) => s.hash(to_hash),
//             SchemeDispatcher::Scheme02(s) => s.hash(to_hash),
//         }
//     }
//
//     fn validate(&self, to_hash: &ContentToHash, pwd_ref: &str) -> Result<()> {
//         match self {
//             SchemeDispatcher::Scheme01(s) => s.validate(to_hash, pwd_ref),
//             SchemeDispatcher::Scheme02(s) => s.validate(to_hash, pwd_ref),
//         }
//     }
// }
//
// // NOTE: We can return a Result of something that impl Scheme (i.e., SchemeDispatcher),
// // and this keeps the SchemeDispatcher private and not exposed outside this module.
// pub fn get_scheme(scheme_name: &str) -> Result<impl Scheme> {
//     match scheme_name {
//         "01" => Ok(SchemeDispatcher::Scheme01(scheme_01::Scheme01)),
//         "02" => Ok(SchemeDispatcher::Scheme02(scheme_02::Scheme02)),
//         _ => Err(Error::SchemeNotFound(scheme_name.to_string())),
//     }
// }
//
// // endregion:    -- Static Dispatch Example (Manual/Vanilla)
