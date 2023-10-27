use std::fmt;

#[derive(Debug, PartialEq)]
pub enum AppError {
    CannotRemoveLastOwner,
    InternalError,
    Unauthorized,
    DatabaseError(String),
    NotAProject,
    NotImplemented
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::CannotRemoveLastOwner => write!(f, "Bad request"),
            AppError::DatabaseError(e) => write!(f, "{}", e),
            AppError::InternalError => write!(f, "Internal error"),
            AppError::NotAProject => write!(f, "Bad request"),
            AppError::NotImplemented => write!(f, "Not implemented"),
            AppError::Unauthorized => write!(f, "Unauthorized")
        }
    }
}
