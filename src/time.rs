use chrono::{DateTime, SecondsFormat, Utc};

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
        .to_rfc3339_opts(SecondsFormat::AutoSi, true)
    )
}

pub fn rfc3339_to_nanos(s: &str) -> Result<i64, Error> {
    let dt = s.parse::<DateTime<Utc>>()?;
    dt.timestamp_nanos_opt()
        .ok_or(Error::OutOfRangeDateTime(dt))
}

mod test {
    #[test]
    fn nanos_to_rfc3339_too_small() {
        assert!(nanos_to_rfc3339(-1).is_err());
    }

    #[test]
    fn nanos_to_rfc3339_zero() {
        assert_eq!(nanos_to_rfc3339(0).unwrap(), "1970-01-01T00:00:00Z");
    }

    #[test]
    fn nanos_to_rfc3339_max() {
        assert_eq!(nanos_to_rfc3339(i64::MAX).unwrap(), "2262-04-11T23:47:16.854775807Z");
    }

    #[test]
    fn rfc3339_to_nanos_zero() {
        assert_eq!(rfc3339_to_nanos("1970-01-01T00:00:00Z").unwrap(), 0);
    }

    #[test]
    fn rfc3339_to_nanos_too_early() {
        assert!(rfc3339_to_nanos("1677-09-21T00:12:43.145224191Z").is_err());
    }

    #[test]
    fn rfc3339_to_nanos_too_late() {
        assert!(rfc3339_to_nanos("2262-04-11T23:47:16.854775808Z").is_err());
    }
}
