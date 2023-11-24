use crate::{
    query::{Query, QueryError},
    schema::Table,
    Bind, Error, Result,
};

use async_trait::async_trait;
use sqlx::{database::HasArguments, Database, Executor, IntoArguments};

/// Delete rows from a [`sqlx::Database`]
#[async_trait]
pub trait Delete: Table + Bind + Send + Sync + Unpin + 'static {
    /// Delete row in database
    async fn delete<'e, E>(&self, executor: E) -> Result<<crate::Driver as Database>::QueryResult>
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

    // Delete all rows in the list of primary keys
    //async fn delete_many<'e, E>(pks: &[impl AsRef<Self::PrimaryKey>], executor: E) -> Result<()>
    //where
    //Self: Bind<sqlx::Postgres> + Sync + 'static,
    //E: sqlx::Executor<'e, Database = sqlx::Postgres>,
    //for<'q> <sqlx::Postgres as sqlx::database::HasArguments<'q>>::Arguments:
    //Send + sqlx::IntoArguments<'q, sqlx::Postgres>;
}

#[async_trait]
impl<T> Delete for T
where
    T: Table + Bind + Send + Sync + Unpin + 'static,
{
    async fn delete<'e, E>(&self, executor: E) -> Result<<crate::Driver as Database>::QueryResult>
    where
        E: Executor<'e, Database = crate::Driver>,
        for<'q> <crate::Driver as HasArguments<'q>>::Arguments:
            IntoArguments<'q, crate::Driver> + Send,
    {
        let Query {
            builder, bindings, ..
        } = crate::runtime::sql::delete::<T>();

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

    async fn delete_by<'e, E>(
        pk: &Self::PrimaryKey,
        executor: E,
    ) -> Result<<crate::Driver as Database>::QueryResult>
    where
        E: Executor<'e, Database = crate::Driver>,
        for<'q> <crate::Driver as HasArguments<'q>>::Arguments:
            IntoArguments<'q, crate::Driver> + Send,
    {
        let Query {
            builder, bindings, ..
        } = crate::runtime::sql::delete::<T>();

        assert!(bindings.columns().len() == 1);
        assert!(bindings.columns()[0].field() == Self::PRIMARY_KEY.field);
        assert!(bindings.columns()[0].sql() == Self::PRIMARY_KEY.sql);

        let query = sqlx::query(builder.sql()).bind(pk).persistent(false);

        query
            .execute(executor)
            .await
            .map_err(QueryError::from)
            .map_err(Error::Query)
    }
}
