use thiserror::Error;

use crate::core::CoreError;

// TODO: better error messsages
#[derive(Debug, Error, PartialEq)]
pub enum AppError {
    #[error("Unsupported media type")]
    BadMimeType,
//    #[error("Cannot remove last project owner")]
    #[error("Bad request")]
    CannotRemoveLastOwner,
    #[error("{0}")]
    DatabaseError(String),
// TODO: Internal error should have a string? cause?
    #[error("Internal error")]
    InternalError,
    #[error("Unprocessable entity")]
    JsonError,
    #[error("Bad request")]
    LimitOutOfRange,
    #[error("Bad request")]
    MalformedQuery,
    #[error("Bad request")]
    MalformedVersion,
    #[error("Not found")]
    NotAUser,
    #[error("Not found")]
    NotFound,
    #[error("Unauthorized")]
    Unauthorized
}

impl From<CoreError> for AppError {
    fn from(err: CoreError) -> Self {
        match err {
            CoreError::CannotRemoveLastOwner => AppError::CannotRemoveLastOwner  ,
            CoreError::InvalidProjectName => AppError::MalformedQuery, // FIXME
            CoreError::ProjectNameInUse => AppError::MalformedQuery, // FIXME
            CoreError::MalformedQuery => AppError::MalformedQuery,
            CoreError::NotFound => AppError::NotFound,
            CoreError::NotAPackage => AppError::NotFound,
            CoreError::NotAProject => AppError::NotFound,
            CoreError::NotARevision => AppError::NotFound,
            CoreError::NotAUser => AppError::NotAUser,
            CoreError::NotAVersion => AppError::NotFound,
            CoreError::InternalError => AppError::InternalError,
            CoreError::DatabaseError(e) => AppError::DatabaseError(e.to_string()),
            CoreError::TimeError(_) => AppError::InternalError,
            CoreError::SeekError(_) => AppError::InternalError
        }
    }
}
