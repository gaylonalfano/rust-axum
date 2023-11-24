// region:       -- Modules
mod error;

pub use self::error::{Error, Result};

use lazy_regex::regex::Replacer;
use time::format_description::well_known::Rfc3339;
use time::{Duration, OffsetDateTime};
// endregion:    -- Modules

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

// region:       -- Base64Url
pub fn b64u_encode(content: &str) -> String {
    base64_url::encode(content)
}

pub fn b64u_decode(b64u: &str) -> Result<String> {
    // NOTE: We don't care about the Error so much. We just want to
    // know if it fails.
    let decoded_string = base64_url::decode(b64u)
        .ok()
        .and_then(|r| String::from_utf8(r).ok())
        .ok_or(Error::FailToB64uDecode)?;

    Ok(decoded_string)
}
// endregion:    -- Base64Url
