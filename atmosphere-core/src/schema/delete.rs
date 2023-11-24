use crate::{hooks::Hooks, query::QueryError, schema::Table, Bind, Error, Result};

use async_trait::async_trait;
use sqlx::{database::HasArguments, Database, Executor, IntoArguments};

/// Delete rows from a [`sqlx::Database`]
#[async_trait]
pub trait Delete: Table + Bind + Hooks + Send + Sync + Unpin + 'static {
    /// Delete row in database
    async fn delete<'e, E>(
        &mut self,
        executor: E,
    ) -> Result<<crate::Driver as Database>::QueryResult>
    where
        E: Executor<'e, Database = crate::Driver>,
        for<'q> <crate::Driver as HasArguments<'q>>::Arguments:
            IntoArguments<'q, crate::Driver> + Send;

    /// Delete row in database by primary key
    async fn delete_by<'e, E>(
        pk: &Self::PrimaryKey,
        executor: E,
    ) -> Result<<crate::Driver as Database>::QueryResult>
    where
        E: Executor<'e, Database = crate::Driver>,
        for<'q> <crate::Driver as HasArguments<'q>>::Arguments:
            IntoArguments<'q, crate::Driver> + Send;
}

#[async_trait]
impl<T> Delete for T
where
    T: Table + Bind + Hooks + Send + Sync + Unpin + 'static,
{
    async fn delete<'e, E>(
        &mut self,
        executor: E,
    ) -> Result<<crate::Driver as Database>::QueryResult>
    where
        E: Executor<'e, Database = crate::Driver>,
        for<'q> <crate::Driver as HasArguments<'q>>::Arguments:
            IntoArguments<'q, crate::Driver> + Send,
    {
        let query = crate::runtime::sql::delete::<T>();

        self.validate(&query)?;
        self.prepare(&query)?;

        Self::inspect(&query);

        let mut sql = sqlx::query(query.sql());

        for c in query.bindings.columns() {
            sql = self.bind(c, sql).unwrap();
        }

        sql.persistent(false)
            .execute(executor)
            .await
            .map_err(QueryError::from)
            .map_err(Error::Query)
    }

    async fn delete_by<'e, E>(
        pk: &Self::PrimaryKey,
        executor: E,
    ) -> Result<<crate::Driver as Database>::QueryResult>
    where
        E: Executor<'e, Database = crate::Driver>,
        for<'q> <crate::Driver as HasArguments<'q>>::Arguments:
            IntoArguments<'q, crate::Driver> + Send,
    {
        let query = crate::runtime::sql::delete::<T>();

        Self::inspect(&query);

        assert!(query.bindings().columns().len() == 1);
        assert!(query.bindings().columns()[0].field() == Self::PRIMARY_KEY.field);
        assert!(query.bindings().columns()[0].sql() == Self::PRIMARY_KEY.sql);

        sqlx::query(query.sql())
            .bind(pk)
            .persistent(false)
            .execute(executor)
            .await
            .map_err(QueryError::from)
            .map_err(Error::Query)
    }
}
