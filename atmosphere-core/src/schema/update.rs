use crate::{hooks::Hooks, query::QueryError, schema::Table, Bind, Error, Result};

use async_trait::async_trait;
use sqlx::{database::HasArguments, Database, Executor, IntoArguments};

/// Update rows in the database
#[async_trait]
pub trait Update: Table + Bind + Hooks + Send + Sync + Unpin + 'static {
    /// Update the row in the database
    async fn update<'e, E>(
        &mut self,
        executor: E,
    ) -> Result<<crate::Driver as Database>::QueryResult>
    where
        E: Executor<'e, Database = crate::Driver>,
        for<'q> <crate::Driver as HasArguments<'q>>::Arguments:
            IntoArguments<'q, crate::Driver> + Send;

    /// Save to the database
    async fn save<'e, E>(
        &mut self,
        executor: E,
    ) -> Result<<crate::Driver as Database>::QueryResult>
    where
        E: Executor<'e, Database = crate::Driver>,
        for<'q> <crate::Driver as HasArguments<'q>>::Arguments:
            IntoArguments<'q, crate::Driver> + Send;
}

#[async_trait]
impl<T> Update for T
where
    T: Table + Bind + Hooks + Send + Sync + Unpin + 'static,
{
    async fn update<'e, E>(
        &mut self,
        executor: E,
    ) -> Result<<crate::Driver as Database>::QueryResult>
    where
        E: Executor<'e, Database = crate::Driver>,
        for<'q> <crate::Driver as HasArguments<'q>>::Arguments:
            IntoArguments<'q, crate::Driver> + Send,
    {
        let query = crate::runtime::sql::update::<T>();

        self.validate(&query)?;
        self.prepare(&query)?;

        Self::inspect(&query);

        let mut sql = sqlx::query(query.sql());

        for c in query.bindings().columns() {
            sql = self.bind(c, sql).unwrap();
        }

        let result = sql
            .persistent(false)
            .execute(executor)
            .await
            .map_err(QueryError::from)
            .map_err(Error::Query)?;

        Ok(result)
    }

    async fn save<'e, E>(&mut self, executor: E) -> Result<<crate::Driver as Database>::QueryResult>
    where
        E: Executor<'e, Database = crate::Driver>,
        for<'q> <crate::Driver as HasArguments<'q>>::Arguments:
            IntoArguments<'q, crate::Driver> + Send,
    {
        let query = crate::runtime::sql::upsert::<T>();

        self.validate(&query)?;
        self.prepare(&query)?;

        Self::inspect(&query);

        let mut sql = sqlx::query(query.sql());

        for c in query.bindings().columns() {
            sql = self.bind(c, sql).unwrap();
        }

        let result = sql
            .persistent(false)
            .execute(executor)
            .await
            .map_err(QueryError::from)
            .map_err(Error::Query)?;

        Ok(result)
    }
}
