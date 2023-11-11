use crate::{Column, Result, Table};
use sqlx::database::HasArguments;
use sqlx::query::QueryAs;
use sqlx::{Database, Encode, QueryBuilder, Type};

type Query<'q, DB> = sqlx::query::Query<'q, DB, <DB as HasArguments<'q>>::Arguments>;

/// Bindable query abstraction. Implementors are [`sqlx::query::Query`] & [`sqlx::query::QueryAs`];
pub trait Bindable<'q, DB>
where
    DB: Database + for<'a> HasArguments<'a>,
{
    fn dyn_bind<T: 'q + Send + Encode<'q, DB> + Type<DB>>(self, value: T) -> Self;
}

impl<'q, DB> Bindable<'q, DB> for Query<'q, DB>
where
    DB: Database + for<'a> HasArguments<'a>,
{
    fn dyn_bind<T: 'q + Send + Encode<'q, DB> + Type<DB>>(self, value: T) -> Self {
        self.bind(value)
    }
}

impl<'q, E, DB> Bindable<'q, DB> for QueryAs<'q, DB, E, <DB as HasArguments<'q>>::Arguments>
where
    DB: Database + for<'a> HasArguments<'a>,
{
    fn dyn_bind<T: 'q + Send + Encode<'q, DB> + Type<DB>>(self, value: T) -> Self {
        self.bind(value)
    }
}

impl<'q, DB> Bindable<'q, DB> for QueryBuilder<'q, DB>
where
    DB: Database + for<'a> HasArguments<'a>,
{
    fn dyn_bind<T: 'q + Send + Encode<'q, DB> + Type<DB>>(mut self, value: T) -> Self {
        self.push_bind(value);
        self
    }
}

/// Bind columns to SQL Queries
pub trait Bind: Table {
    /// Bind a single column to the query
    fn bind<'q, Q: Bindable<'q, Self::Database>>(
        &'q self,
        c: &'q Column<Self>,
        query: Q,
    ) -> Result<Q>;
}
