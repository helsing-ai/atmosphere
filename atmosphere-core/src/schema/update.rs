use crate::{
    query::{Query, QueryError},
    schema::Table,
    Bind, Error, Result,
};

use async_trait::async_trait;
use sqlx::{database::HasArguments, Database, Executor, IntoArguments};

/// Update rows in a [`sqlx::Database`]
#[async_trait]
pub trait Update: Table + Bind + Send + Sync + Unpin + 'static {
    /// Update the row in the database
    async fn update<'e, E>(&self, executor: E) -> Result<<crate::Driver as Database>::QueryResult>
    where
        E: Executor<'e, Database = crate::Driver>,
        for<'q> <crate::Driver as HasArguments<'q>>::Arguments:
            IntoArguments<'q, crate::Driver> + Send;

    /// Save to the database
    async fn save<'e, E>(&self, executor: E) -> Result<<crate::Driver as Database>::QueryResult>
    where
        E: Executor<'e, Database = crate::Driver>,
        for<'q> <crate::Driver as HasArguments<'q>>::Arguments:
            IntoArguments<'q, crate::Driver> + Send;
}

#[async_trait]
impl<T> Update for T
where
    T: Table + Bind + Send + Sync + Unpin + 'static,
{
    async fn update<'e, E>(&self, executor: E) -> Result<<crate::Driver as Database>::QueryResult>
    where
        E: Executor<'e, Database = crate::Driver>,
        for<'q> <crate::Driver as HasArguments<'q>>::Arguments:
            IntoArguments<'q, crate::Driver> + Send,
    {
        let Query {
            builder, bindings, ..
        } = crate::runtime::sql::update::<T>();

        let mut query = sqlx::query(builder.sql());

        for c in bindings.columns() {
            query = self.bind(c, query).unwrap();
        }

        query
            .persistent(false)
            .execute(executor)
            .await
            .map_err(QueryError::from)
            .map_err(Error::Query)
    }

    async fn save<'e, E>(&self, executor: E) -> Result<<crate::Driver as Database>::QueryResult>
    where
        E: Executor<'e, Database = crate::Driver>,
        for<'q> <crate::Driver as HasArguments<'q>>::Arguments:
            IntoArguments<'q, crate::Driver> + Send,
    {
        let Query {
            builder, bindings, ..
        } = crate::runtime::sql::upsert::<T>();

        let mut query = sqlx::query(builder.sql());

        for c in bindings.columns() {
            query = self.bind(c, query).unwrap();
        }

        query
            .persistent(false)
            .execute(executor)
            .await
            .map_err(QueryError::from)
            .map_err(Error::Query)
    }
}
