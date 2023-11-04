use crate::{Column, Result, Table};
use sqlx::query::QueryAs;
use sqlx::Encode;
use sqlx::Type;
use sqlx::{database::HasArguments, Database};

type Query<'q, DB> = sqlx::query::Query<'q, DB, <DB as HasArguments<'q>>::Arguments>;

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

/// Bind columns to SQL Queries
pub trait Bind<DB: Database>: Table
where
    DB: Database,
{
    /// Bind a single column to the query
    fn bind<'q, Q: Bindable<'q, DB>>(&'q self, c: &'q Column<Self>, query: Q) -> Result<Q>;

    /// Bind a all columns to the query
    fn bind_all<'q, Q: Bindable<'q, DB>>(&'q self, mut query: Q) -> Result<Q> {
        query = self.bind_primary_key(query)?;
        query = self.bind_foreign_keys(query)?;
        query = self.bind_data(query)?;

        Ok(query)
    }

    /// Bind the primary key column to the query
    fn bind_primary_key<'q, Q: Bindable<'q, DB>>(&'q self, query: Q) -> Result<Q> {
        self.bind(&Self::PRIMARY_KEY, query)
    }

    /// Bind the foreign keys columns to the query
    fn bind_foreign_keys<'q, Q: Bindable<'q, DB>>(&'q self, mut query: Q) -> Result<Q> {
        for ref fk in Self::FOREIGN_KEYS {
            query = self.bind(fk, query)?;
        }

        Ok(query)
    }

    /// Bind the data columns to the query
    fn bind_data<'q, Q: Bindable<'q, DB>>(&'q self, mut query: Q) -> Result<Q> {
        for ref data in Self::DATA {
            query = self.bind(data, query)?;
        }

        Ok(query)
    }
}
