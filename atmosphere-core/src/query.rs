use sqlx::QueryBuilder;
use thiserror::Error;

use crate::{runtime::sql::Bindings, Bind};

/// An error that occured while executing the query
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum QueryError {
    /// Database communication (IO / Protocol / TLS) related errors
    #[error("IO")]
    Io(#[source] sqlx::Error),

    /// Row not found errors
    #[error("not found")]
    NotFound(#[source] sqlx::Error),

    /// SQLSTATE errors
    #[error("sql")]
    Sql(#[source] SqlError),

    /// Violation errors
    #[error("violation")]
    Violation(#[source] ViolationError),

    /// Catch-all for sqlx errors
    #[error("sqlx")]
    Other(#[source] sqlx::Error),

    /// Atmosphere internal error
    #[error("internal error")]
    InternalError(#[source] sqlx::Error),
}

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ViolationError {
    #[error("uniqueness violation")]
    Unique(#[source] sqlx::Error),
    #[error("foreign key violation")]
    ForeignKey(#[source] sqlx::Error),
    #[error("integrity check")]
    Check(#[source] sqlx::Error),
}

/// SQLSTATE derived errors
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum SqlError {
    /// SQLSTATE Class 22
    #[error("data exception")]
    DataException(#[source] sqlx::Error),

    /// SQLSTATE Class 23
    #[error("integrity constraint")]
    IntegrityConstraint(#[source] sqlx::Error),

    /// SQLSTATE Class 42
    #[error("syntax")]
    Syntax(#[source] sqlx::Error),

    /// All other classes
    #[error("other")]
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

/// Cardinality information on the affected rows
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cardinality {
    None,
    One,
    Many,
}

/// The operation that the query contains
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Operation {
    Select,
    Insert,
    Update,
    Upsert,
    Delete,
    Other,
}

/// A generated atmosphere query over a given table
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

    pub const fn bindings(&self) -> &Bindings<T> {
        &self.bindings
    }
}
