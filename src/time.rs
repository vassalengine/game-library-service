use chrono::{DateTime, Utc};

use crate::errors::AppError;

// FIXME: do we want some other error type for these?
pub fn nanos_to_rfc3339(ns: i64) -> Result<String, AppError> {
    Ok(
        DateTime::<Utc>::from_timestamp(
            (ns / 1_000_000_000) as i64,
            (ns % 1_000_000_000) as u32
        )
        .ok_or(AppError::InternalError)?
        .to_rfc3339()
    )
}

pub fn rfc3339_to_nanos(s: &str) -> Result<i64, AppError> {
    s.parse::<DateTime<Utc>>()
        .or(Err(AppError::MalformedQuery))?
        .timestamp_nanos_opt()
        .ok_or(AppError::MalformedQuery)
}
