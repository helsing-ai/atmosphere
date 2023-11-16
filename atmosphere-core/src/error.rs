use thiserror::Error;

use crate::{query::QueryError, BindError};

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    #[error("io")]
    Io(#[from] std::io::Error),

    #[error("query")]
    Query(#[from] QueryError),

    #[error("bind")]
    Bind(#[from] BindError),

    #[error("other")]
    Other,

    #[error("internal")]
    Internal,
}

/// A result type for atmosphere code
pub type Result<T> = std::result::Result<T, Error>;
