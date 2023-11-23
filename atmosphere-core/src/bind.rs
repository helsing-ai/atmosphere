use crate::{Column, Result, Table};
use sqlx::database::HasArguments;
use sqlx::query::QueryAs;
use sqlx::{Encode, QueryBuilder, Type};
use thiserror::Error;

/// An error that occured while binding values
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum BindError {
    #[error("unknown column: {0}")]
    Unknown(&'static str),
}

type Query<'q, DB> = sqlx::query::Query<'q, DB, <DB as HasArguments<'q>>::Arguments>;

/// Bindable query abstraction. Implementors are [`sqlx::query::Query`] & [`sqlx::query::QueryAs`];
pub trait Bindable<'q> {
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
