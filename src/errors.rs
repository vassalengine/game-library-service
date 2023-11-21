use std::fmt;

#[derive(Debug, PartialEq)]
pub enum AppError {
    BadMimeType,
    CannotRemoveLastOwner,
    InternalError,
    Unauthorized,
    DatabaseError(String),
    JsonError,
    LimitOutOfRange,
    MalformedQuery,
    MalformedVersion,
    NotAPackage,
    NotAProject,
    NotARevision,
    NotAVersion,
    NotImplemented
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::BadMimeType => write!(f, "Unsupported media type"),
            AppError::CannotRemoveLastOwner => write!(f, "Bad request"),
            AppError::DatabaseError(e) => write!(f, "{}", e),
            AppError::InternalError => write!(f, "Internal error"),
            AppError::LimitOutOfRange => write!(f, "Bad request"),
            AppError::JsonError => write!(f, "Unprocessable entity"),
            AppError::MalformedQuery => write!(f, "Bad request"),
            AppError::MalformedVersion => write!(f, "Bad request"),
            AppError::NotAPackage => write!(f, "Bad request"),
            AppError::NotAProject => write!(f, "Bad request"),
            AppError::NotARevision => write!(f, "Bad request"),
            AppError::NotAVersion => write!(f, "Bad request"),
            AppError::NotImplemented => write!(f, "Not implemented"),
            AppError::Unauthorized => write!(f, "Unauthorized")
        }
    }
}
