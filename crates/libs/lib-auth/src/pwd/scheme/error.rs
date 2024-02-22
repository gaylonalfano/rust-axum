use serde::Serialize;

pub type Result<T> = core::result::Result<T, Error>;

// NOTE: Again, for the RequestLogLine we're using JSON serialization
#[derive(Debug, Serialize)]
pub enum Error {
    Key,
    Salt,
    Hash,
    PwdValidate,
    SchemeNotFound(String), // We'll store the attempted scheme
}

// region:    --- Error Boilerplate
impl core::fmt::Display for Error {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}

impl std::error::Error for Error {}
// endregion: --- Error Boilerplate
