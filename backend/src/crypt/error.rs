use serde::Serialize;

pub type Result<T> = core::result::Result<T, Error>;

// NOTE: Because this crypt error may cascade UP to the
// web layer and used in RequestLogLine, it needs Serialize
// to serialize into the JSON data format.
#[derive(Debug, Serialize)]
pub enum Error {
    // -- Key
    KeyFailHmac,

    // -- Pwd
    PwdNotMatching,

    // -- Token
    TokenInvalidFormat,
    TokenCannotDecodeIdent,
    TokenCannotDecodeExp,
    TokenSignatureNotMatching,
    TokenExpNotIso,
    TokenExpired,
}

// region: -- Error Boilerplate
impl core::fmt::Display for Error {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}

impl std::error::Error for Error {}
// endregion: -- Error Boilerplate
