use thiserror::Error;

use crate::core::{AddImageError, AddFileError, AddPlayerError, CoreError, CreatePackageError, CreateProjectError, CreateReleaseError, GetIdError, GetImageError, GetPlayersError, GetProjectError, GetProjectsError, RemovePlayerError, UpdateProjectError, UserIsOwnerError};

// TODO: better error messsages
#[derive(Debug, Error, PartialEq)]
pub enum AppError {
    #[error("Unsupported media type")]
    BadMimeType,
    #[error("Payload too large")]
    TooLarge,
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
            CoreError::NotAUser => AppError::NotAUser,
            CoreError::XDatabaseError(e) => AppError::DatabaseError(e.to_string())
        }
    }
}

impl From<GetIdError> for AppError {
    fn from(err: GetIdError) -> Self {
        match err {
            GetIdError::DatabaseError(e) => AppError::DatabaseError(e.to_string()),
            GetIdError::NotFound => AppError::NotFound
        }
    }
}

impl From<UserIsOwnerError> for AppError {
    fn from(err: UserIsOwnerError) -> Self {
        match err {
            UserIsOwnerError::DatabaseError(e) => AppError::DatabaseError(e.to_string())
        }
    }
}

impl From<GetProjectsError> for AppError {
    fn from(err: GetProjectsError) -> Self {
        match err {
            GetProjectsError::DatabaseError(e) => AppError::DatabaseError(e.to_string()),
            GetProjectsError::MalformedQuery => AppError::MalformedQuery,
            GetProjectsError::SeekError(_) => AppError::InternalError,
            GetProjectsError::TimeError(_) => AppError::InternalError
        }
    }
}

impl From<GetProjectError> for AppError {
    fn from(err: GetProjectError) -> Self {
        match err {
            GetProjectError::DatabaseError(e) => AppError::DatabaseError(e.to_string()),
            GetProjectError::NotFound => AppError::NotFound,
            GetProjectError::TimeError(_) => AppError::InternalError
        }
    }
}

impl From<CreateProjectError> for AppError {
    fn from(err: CreateProjectError) -> Self {
        match err {
            CreateProjectError::DatabaseError(e) => AppError::DatabaseError(e.to_string()),
            CreateProjectError::TimeError(_) => AppError::InternalError
        }
    }
}

impl From<UpdateProjectError> for AppError {
    fn from(err: UpdateProjectError) -> Self {
        match err {
            UpdateProjectError::DatabaseError(e) => AppError::DatabaseError(e.to_string()),
            UpdateProjectError::TimeError(_) => AppError::InternalError
        }
    }
}

impl From<CreatePackageError> for AppError {
    fn from(err: CreatePackageError) -> Self {
        match err {
            CreatePackageError::DatabaseError(e) => AppError::DatabaseError(e.to_string()),
            CreatePackageError::TimeError(_) => AppError::InternalError
        }
    }
}

impl From<CreateReleaseError> for AppError {
    fn from(err: CreateReleaseError) -> Self {
        match err {
            CreateReleaseError::DatabaseError(e) => AppError::DatabaseError(e.to_string()),
            CreateReleaseError::TimeError(_) => AppError::InternalError
        }
    }
}

impl From<AddFileError> for AppError {
    fn from(err: AddFileError) -> Self {
        match err {
            AddFileError::DatabaseError(e) => AppError::DatabaseError(e.to_string()),
            AddFileError::InvalidFilename => AppError::MalformedQuery,
            AddFileError::IOError(_) => AppError::InternalError,
            AddFileError::ModuleError(e) => AppError::ModuleError(e.to_string()),
            AddFileError::TimeError(_) => AppError::InternalError,
            AddFileError::TooLarge => AppError::TooLarge,
            AddFileError::UploadError(e) => AppError::UploadError(e.to_string())
        }
    }
}

impl From<GetPlayersError> for AppError {
    fn from(err: GetPlayersError) -> Self {
        match err {
            GetPlayersError::DatabaseError(e) => AppError::DatabaseError(e.to_string())
        }
    }
}

impl From<AddPlayerError> for AppError {
    fn from(err: AddPlayerError) -> Self {
        match err {
            AddPlayerError::DatabaseError(e) => AppError::DatabaseError(e.to_string())
        }
    }
}

impl From<RemovePlayerError> for AppError {
    fn from(err: RemovePlayerError) -> Self {
        match err {
            RemovePlayerError::DatabaseError(e) => AppError::DatabaseError(e.to_string())
        }
    }
}

impl From<GetImageError> for AppError {
    fn from(err: GetImageError) -> Self {
        match err {
            GetImageError::DatabaseError(e) => AppError::DatabaseError(e.to_string()),
            GetImageError::NotFound => AppError::NotFound
        }
    }
}

impl From<AddImageError> for AppError {
    fn from(err: AddImageError) -> Self {
        match err {
            AddImageError::BadMimeType => AppError::BadMimeType,
            AddImageError::DatabaseError(e) => AppError::DatabaseError(e.to_string()),
            AddImageError::InvalidFilename => AppError::MalformedQuery,
            AddImageError::IOError(_) => AppError::InternalError,
            AddImageError::TimeError(_) => AppError::InternalError,
            AddImageError::TooLarge => AppError::TooLarge,
            AddImageError::UploadError(e) => AppError::UploadError(e.to_string())
        }
    }
}
