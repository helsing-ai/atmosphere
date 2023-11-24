use crate::{hooks::Hooks, query::QueryError, schema::Table, Bind, Error, Result};

use async_trait::async_trait;
use sqlx::{database::HasArguments, Executor, IntoArguments};

/// Create rows in a [`sqlx::Database`]
#[async_trait]
pub trait Create: Table + Bind + Hooks + Sync + 'static {
    /// Create a new row
    async fn create<'e, E>(
        &mut self,
        executor: E,
    ) -> Result<<crate::Driver as sqlx::Database>::QueryResult>
    where
        E: Executor<'e, Database = crate::Driver>,
        for<'q> <crate::Driver as HasArguments<'q>>::Arguments:
            IntoArguments<'q, crate::Driver> + Send;
}

#[async_trait]
impl<T> Create for T
where
    T: Table + Bind + Hooks + Sync + 'static,
{
    async fn create<'e, E>(
        &mut self,
        executor: E,
    ) -> Result<<crate::Driver as sqlx::Database>::QueryResult>
    where
        E: Executor<'e, Database = crate::Driver>,
        for<'q> <crate::Driver as HasArguments<'q>>::Arguments:
            IntoArguments<'q, crate::Driver> + Send,
    {
        let query = crate::runtime::sql::insert::<T>();

        self.validate(&query)?;
        self.prepare(&query)?;

        Self::inspect(&query);

        let mut builder = sqlx::query(query.sql());

        for c in query.bindings().columns() {
            builder = self.bind(c, builder).unwrap();
        }

        builder
            .persistent(false)
            .execute(executor)
            .await
            .map_err(QueryError::from)
            .map_err(Error::Query)
    }
}
