//! Error Handling Module for Atmosphere
//!
//! This module defines the error handling mechanisms used throughout the Atmosphere framework. It
//! provides a comprehensive `Error` type that encapsulates various kinds of errors that may occur
//! during database operations, file IO, and other framework-related activities.
//!
//! The module simplifies error management by categorizing common error types and providing a
//! unified interface for handling them. This approach enhances code readability and
//! maintainability, especially in scenarios involving complex database interactions and
//! operations.

use miette::Diagnostic;
use thiserror::Error;

use crate::{BindError, query::QueryError};

/// Errors that can occur within Atmosphere.
///
/// This enum encapsulates a range of errors including IO errors, query-related errors, binding
/// errors, and others. It is designed to provide a unified error handling mechanism across
/// different components of the framework.
#[derive(Debug, Diagnostic, Error)]
#[non_exhaustive]
pub enum Error {
    #[error("io")]
    #[diagnostic(code(atmosphere::io))]
    Io(#[from] std::io::Error),

    #[error("query")]
    #[diagnostic(transparent)]
    Query(#[from] QueryError),

    #[error("bind")]
    #[diagnostic(transparent)]
    Bind(#[from] BindError),

    #[error("other")]
    #[diagnostic(code(atmosphere::other))]
    Other,

    #[error("internal")]
    #[diagnostic(code(atmosphere::internal))]
    Internal,
}

/// A specialized `Result` type for use throughout the Atmosphere framework.
///
/// This type alias simplifies error handling by using the `Error` enum as the default error type.
/// It is used as the return type for functions and methods within the framework, where errors are
/// expected to be one of the variants defined in the `Error` enum.
pub type Result<T> = std::result::Result<T, Error>;
