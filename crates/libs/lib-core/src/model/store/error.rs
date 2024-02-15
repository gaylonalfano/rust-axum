use serde::Serialize;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Serialize)]
pub enum Error {
    // Eventually we'll use sqlx and sqlb for errors (I think...)
    FailToCreatePool(String),
}

// region: -- Error Boilerplate
// NOTE: This ultimately helps our BACKEND services Request Log Lines
// since they're going to be in JSON format. We use serde Serialize
// to JSON, not because we send to Client, but because we can then
// push these formatted errors cloudwatch or other logging services.
// REF: https://youtu.be/3cA_mk4vdWY?t=2610
// NOTE: Jeremy likes 'thiserror' for command line formatting
impl core::fmt::Display for Error {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}

// Allow us to use '?' for our custom store error
impl std::error::Error for Error {}
// endregion: -- Error Boilerplate
