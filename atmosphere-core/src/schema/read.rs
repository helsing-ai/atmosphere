use crate::{
    hooks::{self, HookInput, HookStage, Hooks},
    query::{QueryError, QueryResult},
    schema::Table,
    Bind, Error, Result,
};

use async_trait::async_trait;
use sqlx::{database::HasArguments, Executor, IntoArguments};

/// Trait for reading rows from a database.
///
/// This trait provides the functionality for reading data from tables in a SQL database. It
/// defines several asynchronous methods for retrieving rows either by their primary key, reloading
/// existing entities, or fetching all rows in a table. The trait incorporates hooks at various
/// stages, allowing for custom logic to be executed as part of the reading process.
#[async_trait]
pub trait Read: Table + Bind + Hooks + Send + Sync + Unpin + 'static {
    /// Finds and retrieves a row by its primary key. This method constructs a query to fetch
    /// a single row based on the primary key, executes it, and returns the result, optionally
    /// triggering hooks before and after execution.
    async fn find<'e, E>(pk: &Self::PrimaryKey, executor: E) -> Result<Option<Self>>
    where
        E: Executor<'e, Database = crate::Driver>,
        for<'q> <crate::Driver as HasArguments<'q>>::Arguments:
            IntoArguments<'q, crate::Driver> + Send;

    /// Retrieves all rows from the table. This method is useful for fetching the complete
    /// dataset of a table, executing a query to return all rows, and applying hooks as needed.
    async fn find_all<'e, E>(executor: E) -> Result<Vec<Self>>
    where
        E: Executor<'e, Database = crate::Driver>,
        for<'q> <crate::Driver as HasArguments<'q>>::Arguments:
            IntoArguments<'q, crate::Driver> + Send;

    /// Reloads the current entity from the database. This method is designed to update the entity
    /// instance with the latest data from the database, ensuring that it reflects the current
    /// state of the corresponding row.
    async fn reload<'e, E>(&mut self, executor: E) -> Result<()>
    where
        E: Executor<'e, Database = crate::Driver>,
        for<'q> <crate::Driver as HasArguments<'q>>::Arguments:
            IntoArguments<'q, crate::Driver> + Send;

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
    T: Table + Bind + Hooks + Send + Sync + Unpin + 'static,
{
    async fn find<'e, E>(pk: &Self::PrimaryKey, executor: E) -> Result<Option<Self>>
    where
        E: Executor<'e, Database = crate::Driver>,
        for<'q> <crate::Driver as HasArguments<'q>>::Arguments:
            IntoArguments<'q, crate::Driver> + Send,
    {
        let query = crate::runtime::sql::select::<T>();

        hooks::execute(HookStage::PreBind, &query, HookInput::PrimaryKey(pk)).await?;

        assert!(query.bindings().columns().len() == 1);
        assert!(query.bindings().columns()[0].field() == Self::PRIMARY_KEY.field);
        assert!(query.bindings().columns()[0].sql() == Self::PRIMARY_KEY.sql);

        hooks::execute(HookStage::PreExec, &query, HookInput::None).await?;

        let res = sqlx::query_as(query.sql())
            .bind(pk)
            .persistent(false)
            .fetch_optional(executor)
            .await
            .map_err(QueryError::from)
            .map_err(Error::Query);

        hooks::execute(
            hooks::HookStage::PostExec,
            &query,
            QueryResult::Optional(&res).into(),
        )
        .await?;

        res
    }

    async fn reload<'e, E>(&mut self, executor: E) -> Result<()>
    where
        E: Executor<'e, Database = crate::Driver>,
        for<'q> <crate::Driver as HasArguments<'q>>::Arguments:
            IntoArguments<'q, crate::Driver> + Send,
    {
        let query = crate::runtime::sql::select_by::<T>(T::PRIMARY_KEY.as_col());

        hooks::execute(HookStage::PreBind, &query, HookInput::Row(self)).await?;

        let mut sql = sqlx::query_as(query.sql());

        for c in query.bindings().columns() {
            sql = self.bind(c, sql).unwrap();
        }

        hooks::execute(HookStage::PreExec, &query, HookInput::None).await?;

        let res = sql
            .persistent(false)
            .fetch_one(executor)
            .await
            .map_err(QueryError::from)
            .map_err(Error::Query);

        hooks::execute(
            hooks::HookStage::PostExec,
            &query,
            QueryResult::One(&res).into(),
        )
        .await?;

        *self = res?;

        Ok(())
    }

    async fn find_all<'e, E>(executor: E) -> Result<Vec<Self>>
    where
        E: Executor<'e, Database = crate::Driver>,
        for<'q> <crate::Driver as HasArguments<'q>>::Arguments:
            IntoArguments<'q, crate::Driver> + Send,
    {
        let query = crate::runtime::sql::select_all::<T>();

        hooks::execute(HookStage::PreBind, &query, HookInput::None).await?;
        hooks::execute(HookStage::PreExec, &query, HookInput::None).await?;

        let res = sqlx::query_as(query.sql())
            .persistent(false)
            .fetch_all(executor)
            .await
            .map_err(QueryError::from)
            .map_err(Error::Query);

        hooks::execute(
            hooks::HookStage::PostExec,
            &query,
            QueryResult::Many(&res).into(),
        )
        .await?;

        res
    }
}
