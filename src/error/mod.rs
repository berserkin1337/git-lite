use std::error::Error;
use std::{
    fmt::{Display, Formatter},
    path::PathBuf,
};


#[derive(Debug,Clone)]
pub enum GitError {
    UnknownError,
    GenericError(String),
    ObjectError(String),
    PathError(String, PathBuf),
}

impl Display for GitError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            GitError::UnknownError => f.write_str("ERROR: Unknown error"),
            GitError::GenericError(message) => write!(f, "ERROR: {}", message),
            GitError::ObjectError(message) => write!(f, "ERROR: {}", message),
            GitError::PathError(message, path) => write!(
                f,
                "ERROR: {}: {}",
                message,
                path.to_str().unwrap().to_owned()
            ),
        }
    }
}

impl Error for GitError{}