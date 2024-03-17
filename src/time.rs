use chrono::{DateTime, Utc};

#[derive(Debug, thiserror::Error, Eq, PartialEq)]
pub enum Error {
    #[error("{0} is out of range")]
    OutOfRangeNs(i64),
    #[error("{0} is out of range")]
    OutOfRangeDateTime(DateTime<Utc>),
    #[error("{0}")]
    ParseError(#[from] chrono::format::ParseError)
}

pub fn nanos_to_rfc3339(ns: i64) -> Result<String, Error> {
    Ok(
        DateTime::<Utc>::from_timestamp(
            ns / 1_000_000_000,
            (ns % 1_000_000_000) as u32
        )
        .ok_or(Error::OutOfRangeNs(ns))?
        .to_rfc3339()
    )
}

pub fn rfc3339_to_nanos(s: &str) -> Result<i64, Error> {
    let dt = s.parse::<DateTime<Utc>>()?;
    dt.timestamp_nanos_opt()
        .ok_or(Error::OutOfRangeDateTime(dt))
}
