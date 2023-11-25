//! Provides structures and enums for handling and executing SQL queries, along with error
//! handling.
//!
//! This module includes custom error types for different database-related errors, enums for query
//! operations and cardinality, and a struct for building and managing queries for database tables.

use miette::Diagnostic;
use sqlx::QueryBuilder;
use thiserror::Error;

use crate::{runtime::sql::Bindings, Bind, Result, Table};

/// Errors that can occur while executing a database query.
///
/// This enum includes variants for IO errors, row not found errors, SQLSTATE errors, violation
/// errors, and others, allowing for detailed categorization and handling of different database
/// errors.
#[derive(Debug, Diagnostic, Error)]
#[non_exhaustive]
pub enum QueryError {
    /// Database communication (IO / Protocol / TLS) related errors
    #[error("IO")]
    #[diagnostic(code(atmosphere::query::io))]
    Io(#[source] sqlx::Error),

    /// Row not found errors
    #[error("not found")]
    #[diagnostic(code(atmosphere::query::not_found))]
    NotFound(#[source] sqlx::Error),

    /// SQLSTATE errors
    #[error("sql")]
    #[diagnostic(transparent)]
    Sql(#[source] SqlError),

    /// Violation errors
    #[error("violation")]
    #[diagnostic(transparent)]
    Violation(#[source] ViolationError),

    /// Catch-all for sqlx errors
    #[error("sqlx")]
    #[diagnostic(code(atmosphere::query::sqlx))]
    Other(#[source] sqlx::Error),

    /// Atmosphere internal error
    #[error("internal error")]
    #[diagnostic(code(atmosphere::query::internal))]
    InternalError(#[source] sqlx::Error),
}

/// Represents errors related to constraint violations in the database.
///
/// Includes uniqueness violations, foreign key violations, and integrity check errors,
/// encapsulating different types of constraint-related issues that can occur during database
/// operations.
#[derive(Debug, Diagnostic, Error)]
#[non_exhaustive]
pub enum ViolationError {
    /// Row uniqueness violated
    #[error("uniqueness violation")]
    #[diagnostic(code(atmosphere::violation::uniqueness))]
    Unique(#[source] sqlx::Error),

    /// Foreign key violation
    #[error("foreign key violation")]
    #[diagnostic(code(atmosphere::violation::foreign_key))]
    ForeignKey(#[source] sqlx::Error),

    /// Integritry check failed
    #[error("integrity check")]
    #[diagnostic(code(atmosphere::violation::integrity))]
    Check(#[source] sqlx::Error),
}

/// Encapsulates errors derived from SQLSTATE codes.
///
/// This enum categorizes various SQL errors such as data exceptions, integrity constraints, syntax
/// errors, and others, based on their SQLSTATE classification.
#[derive(Debug, Diagnostic, Error)]
#[non_exhaustive]
pub enum SqlError {
    /// SQLSTATE Class 22
    #[error("data exception")]
    #[diagnostic(code(atmosphere::sqlstate::data))]
    DataException(#[source] sqlx::Error),

    /// SQLSTATE Class 23
    #[error("integrity constraint")]
    #[diagnostic(code(atmosphere::sqlstate::integrity))]
    IntegrityConstraint(#[source] sqlx::Error),

    /// SQLSTATE Class 42
    #[error("syntax")]
    #[diagnostic(code(atmosphere::sqlstate::syntax))]
    Syntax(#[source] sqlx::Error),

    /// All other classes
    #[error("other")]
    #[diagnostic(code(atmosphere::sqlstate::other))]
    Other(#[source] sqlx::Error),
}

impl From<sqlx::Error> for QueryError {
    fn from(err: sqlx::Error) -> Self {
        use sqlx::Error as E;

        match err {
            E::RowNotFound => Self::NotFound(err),
            E::Io(_)
            | E::Protocol(_)
            | E::Tls(_)
            | E::Configuration(_)
            | E::PoolTimedOut
            | E::PoolClosed
            | E::WorkerCrashed => Self::Io(err),
            E::Database(ref e) => {
                if e.is_unique_violation() {
                    return Self::Violation(ViolationError::Unique(err));
                }

                if e.is_foreign_key_violation() {
                    return Self::Violation(ViolationError::ForeignKey(err));
                }

                if e.is_check_violation() {
                    return Self::Violation(ViolationError::Check(err));
                }

                // SQLSTATE code handling
                // See https://en.wikipedia.org/wiki/SQLSTATE for reference

                if let Some(c) = e.code() {
                    if c.len() < 5 {
                        return Self::InternalError(err);
                    }

                    return match &c.as_ref()[0..1] {
                        "22" => Self::Sql(SqlError::DataException(err)),
                        "23" => Self::Sql(SqlError::IntegrityConstraint(err)),
                        "42" => Self::Sql(SqlError::Syntax(err)),
                        _ => Self::Sql(SqlError::Other(err)),
                    };
                }

                Self::Other(err)
            }
            _ => Self::Other(err),
        }
    }
}

/// Describes the cardinality of the rows affected by a query.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cardinality {
    None,
    One,
    Many,
}

/// Describes the types of operations that a query performs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Operation {
    Select,
    Insert,
    Update,
    Upsert,
    Delete,
    Other,
}

/// Represents a atmosphere query over a database table.
pub struct Query<T: Bind> {
    pub op: Operation,
    pub cardinality: Cardinality,
    pub(crate) builder: QueryBuilder<'static, crate::Driver>,
    pub(crate) bindings: Bindings<T>,
}

impl<T: Bind> Query<T> {
    pub(crate) fn new(
        op: Operation,
        cardinality: Cardinality,
        builder: QueryBuilder<'static, crate::Driver>,
        bindings: Bindings<T>,
    ) -> Self {
        Self {
            op,
            cardinality,
            builder,
            bindings,
        }
    }

    /// Access the generated sql
    pub fn sql(&self) -> &str {
        self.builder.sql()
    }

    /// Access the column bindings
    pub const fn bindings(&self) -> &Bindings<T> {
        &self.bindings
    }
}

/// Describes possible results of executing a query.
pub enum QueryResult<'t, T: Table + Bind> {
    Execution(&'t Result<<crate::Driver as sqlx::Database>::QueryResult>),
    Optional(&'t Result<Option<T>>),
    One(&'t Result<T>),
    Many(&'t Result<Vec<T>>),
}
