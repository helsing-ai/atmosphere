use crate::{
    query::{Query, QueryError},
    schema::Table,
    Bind, Error, Result,
};

use async_trait::async_trait;
use sqlx::{database::HasArguments, Executor, IntoArguments};

/// Read rows from a [`sqlx::Database`]
#[async_trait]
pub trait Read: Table + Bind + Send + Sync + Unpin + 'static {
    /// Find a row by its primary key
    async fn find<'e, E>(pk: &Self::PrimaryKey, executor: E) -> Result<Option<Self>>
    where
        E: Executor<'e, Database = Self::Database>,
        for<'q> <Self::Database as HasArguments<'q>>::Arguments:
            IntoArguments<'q, Self::Database> + Send;

    /// Reload from database
    async fn reload<'e, E>(&mut self, executor: E) -> Result<()>
    where
        E: Executor<'e, Database = Self::Database>,
        for<'q> <Self::Database as HasArguments<'q>>::Arguments:
            IntoArguments<'q, Self::Database> + Send;

    // Find all rows in the list of primary keys
    //async fn find_many<'e, E>(pks: &[impl AsRef<Self::PrimaryKey>], executor: E) -> Result<Vec<Self>>
    //where
    //Self: Bind<sqlx::Postgres> + Sync + 'static,
    //E: sqlx::Executor<'e, Database = sqlx::Postgres>,
    //for<'q> <sqlx::Postgres as sqlx::database::HasArguments<'q>>::Arguments:
    //Send + sqlx::IntoArguments<'q, sqlx::Postgres>;

    // TODO(mara): figure out streams
    // Read all rows from the database
    //async fn all(pool: &sqlx::PgPool) -> Result<Vec<Self>>;
}

#[async_trait]
impl<T> Read for T
where
    T: Table + Bind + Send + Sync + Unpin + 'static,
{
    async fn find<'e, E>(pk: &Self::PrimaryKey, executor: E) -> Result<Option<Self>>
    where
        E: Executor<'e, Database = Self::Database>,
        for<'q> <Self::Database as HasArguments<'q>>::Arguments:
            IntoArguments<'q, Self::Database> + Send,
    {
        let Query {
            builder, bindings, ..
        } = crate::runtime::sql::select::<T>();

        assert!(bindings.columns().len() == 1);
        assert!(bindings.columns()[0].name() == Self::PRIMARY_KEY.name);

        let query = sqlx::query_as(builder.sql()).bind(pk).persistent(false);

        query
            .fetch_optional(executor)
            .await
            .map_err(QueryError::from)
            .map_err(Error::Query)
    }

    async fn reload<'e, E>(&mut self, executor: E) -> Result<()>
    where
        E: Executor<'e, Database = Self::Database>,
        for<'q> <Self::Database as HasArguments<'q>>::Arguments:
            IntoArguments<'q, Self::Database> + Send,
    {
        let Query {
            builder, bindings, ..
        } = crate::runtime::sql::select::<T>();

        let mut query = sqlx::query_as(builder.sql());

        for c in bindings.columns() {
            query = self.bind(c, query).unwrap();
        }

        *self = query
            .persistent(false)
            .fetch_one(executor)
            .await
            .map_err(QueryError::from)
            .map_err(Error::Query)?;

        Ok(())
    }
}
