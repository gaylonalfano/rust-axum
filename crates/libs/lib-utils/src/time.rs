use time::{Duration, OffsetDateTime};

pub use time::format_description::well_known::Rfc3339;

// region:       -- Time
pub fn now_utc() -> OffsetDateTime {
    OffsetDateTime::now_utc()
}

/// Normalize ISO Rfc3339 profile
pub fn format_time(time: OffsetDateTime) -> String {
    // TODO: Need to check if safe
    time.format(&Rfc3339).unwrap()
}

pub fn now_utc_plus_sec_str(sec: f64) -> String {
    let new_time = now_utc() + Duration::seconds_f64(sec);
    format_time(new_time)
}

pub fn parse_utc(moment: &str) -> Result<OffsetDateTime> {
    // NOTE: We want to return our utils module custom error,
    // so need to map_err()
    OffsetDateTime::parse(moment, &Rfc3339).map_err(|_| Error::DateFailParse(moment.to_string()))
}
// endregion:    -- Time

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    // -- Time
    DateFailParse(String),

    // -- Base64 Url
    // NOTE: Not capturing the inner details so it can't
    // be accidentally logged.
    FailToB64uDecode,
}

// region:       -- Error Boilerplate
impl core::fmt::Display for Error {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}

impl std::error::Error for Error {}
// endregion:    -- Error Boilerplate
