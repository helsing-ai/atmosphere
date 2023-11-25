//! Bind Module for Atmosphere SQL Framework
//!
//! This module provides functionality to bind values to SQL queries in a type-safe and efficient
//! manner. It includes traits and implementations that facilitate the binding of parameters to
//! various SQL query types, ensuring that the queries are correctly formatted and executed against
//! the database.
//!
//! Key components of this module include the `Bindable` trait, which abstracts over different
//! types of queries, allowing for flexible and dynamic binding of values, and the `Bind` trait,
//! which provides an interface for binding columns to SQL queries in the context of a specific
//! table.
//!
//! # Types
//!
//! - `BindError`: An error related to binding operations, such as unknown column errors.
//! - `Bindable`: A trait for abstracting over different query types, providing a method to dynamically bind values.
//! - `Bind`: A trait for binding columns to SQL queries, specific to table entities.
//!
//! The module plays a crucial role in the framework, enabling developers to write database
//! interactions that are both expressive and resilient to errors like incorrect parameter types or
//! missing values.

use crate::{Column, Result, Table};
use sqlx::database::HasArguments;
use sqlx::query::QueryAs;
use sqlx::{Encode, QueryBuilder, Type};
use thiserror::Error;

/// Enumerates errors that can occur during the binding of values to SQL queries.
///
/// This enum covers various issues that might arise when binding parameters, such as referencing
/// unknown columns.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum BindError {
    /// Represents an error where a specified column is unknown or not found.
    #[error("unknown column: {0}")]
    Unknown(&'static str),
}

type Query<'q, DB> = sqlx::query::Query<'q, DB, <DB as HasArguments<'q>>::Arguments>;

/// Trait for dynamic binding of values.
///
/// `Bindable` provides an abstraction over different types of SQL queries, such as
/// `sqlx::query::Query` and `sqlx::query::QueryAs`, allowing for flexible and dynamic binding of
/// values. It is designed to work with various query types and enables the binding of values with
/// different types and constraints.
pub trait Bindable<'q> {
    /// Binds a value to the query. The value must be compatible with the `atmosphere::Driver`.
    fn dyn_bind<T: 'q + Send + Encode<'q, crate::Driver> + Type<crate::Driver>>(
        self,
        value: T,
    ) -> Self;
}

impl<'q> Bindable<'q> for Query<'q, crate::Driver> {
    fn dyn_bind<T: 'q + Send + Encode<'q, crate::Driver> + Type<crate::Driver>>(
        self,
        value: T,
    ) -> Self {
        self.bind(value)
    }
}

impl<'q, E> Bindable<'q>
    for QueryAs<'q, crate::Driver, E, <crate::Driver as HasArguments<'q>>::Arguments>
{
    fn dyn_bind<T: 'q + Send + Encode<'q, crate::Driver> + Type<crate::Driver>>(
        self,
        value: T,
    ) -> Self {
        self.bind(value)
    }
}

impl<'q> Bindable<'q> for QueryBuilder<'q, crate::Driver> {
    fn dyn_bind<T: 'q + Send + Encode<'q, crate::Driver> + Type<crate::Driver>>(
        mut self,
        value: T,
    ) -> Self {
        self.push_bind(value);
        self
    }
}

/// Bind columns to SQL Queries
pub trait Bind: Table {
    /// Bind a single column to the query
    fn bind<'q, Q: Bindable<'q>>(&'q self, c: &'q Column<Self>, query: Q) -> Result<Q>;
}
