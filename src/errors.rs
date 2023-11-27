use std::fmt;

#[derive(Debug, PartialEq)]
pub enum AppError {
    BadMimeType,
    CannotRemoveLastOwner,
    DatabaseError(String),
    InternalError,
    JsonError,
    LimitOutOfRange,
    MalformedQuery,
    MalformedVersion,
    NotAPackage,
    NotAProject,
    NotARevision,
    NotAVersion,
    NotFound,
    NotImplemented,
    Unauthorized
}

// TODO: better error messsages
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
            AppError::NotAPackage => write!(f, "Not found"),
            AppError::NotAProject => write!(f, "Not found"),
            AppError::NotARevision => write!(f, "Not found"),
            AppError::NotAVersion => write!(f, "Not found"),
            AppError::NotFound => write!(f, "Not found"),
            AppError::NotImplemented => write!(f, "Not implemented"),
            AppError::Unauthorized => write!(f, "Unauthorized")
        }
    }
}
