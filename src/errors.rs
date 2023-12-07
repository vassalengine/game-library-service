use thiserror::Error;

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
