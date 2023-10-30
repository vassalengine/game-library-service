use std::fmt;

#[derive(Debug, PartialEq)]
pub enum AppError {
    CannotRemoveLastOwner,
    InternalError,
    Unauthorized,
    DatabaseError(String),
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
            AppError::CannotRemoveLastOwner => write!(f, "Bad request"),
            AppError::DatabaseError(e) => write!(f, "{}", e),
            AppError::InternalError => write!(f, "Internal error"),
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
