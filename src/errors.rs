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
    NotAPackage,
    #[error("Not found")]
    NotAProject,
    #[error("Not found")]
    NotARevision,
    #[error("Not found")]
    NotAUser,
    #[error("Not found")]
    NotAVersion,
    #[error("Not found")]
    NotFound,
    #[error("Not implemented")]
    NotImplemented,
    #[error("Unauthorized")]
    Unauthorized
}

impl From<CoreError> for AppError {
    fn from(err: CoreError) -> Self {
        match err {
            CoreError::CannotRemoveLastOwner => AppError::CannotRemoveLastOwner  ,
            CoreError::MalformedQuery => AppError::MalformedQuery,
            CoreError::NotFound => AppError::NotFound,
            CoreError::NotAPackage => AppError::NotAPackage,
            CoreError::NotAProject => AppError::NotAProject,
            CoreError::NotARevision => AppError::NotARevision,
            CoreError::NotAUser => AppError::NotAUser,
            CoreError::NotAVersion => AppError::NotAVersion,
            CoreError::InternalError => AppError::InternalError,
            CoreError::DatabaseError(e) => AppError::DatabaseError(e.to_string()),
            CoreError::TimeError(_) => AppError::InternalError,
            CoreError::SeekError(_) => AppError::InternalError
        }
    }
}
