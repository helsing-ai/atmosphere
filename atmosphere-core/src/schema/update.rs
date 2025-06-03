use crate::{
    Bind, Error, Result,
    hooks::{self, HookInput, HookStage, Hooks},
    query::{QueryError, QueryResult},
    schema::Table,
};

use async_trait::async_trait;
use sqlx::{Database, Executor, IntoArguments};

/// Update rows in a database.
///
/// Provides functionality for updating data in tables within a SQL database. This trait defines
/// asynchronous methods for modifying existing rows in the database, either through direct updates
/// or upserts (update or insert if not exists). It ensures that hooks are executed at various
/// stages, enabling custom logic to be integrated into the update process.
#[async_trait]
pub trait Update: Table + Bind + Hooks + Send + Sync + Unpin + 'static {
    /// Updates an existing row in the database. This method constructs an update query, binds the
    /// necessary values, executes the query, and applies hooks at predefined stages (e.g., before
    /// binding, before execution, after execution).
    async fn update<'e, E>(
        &mut self,
        executor: E,
    ) -> Result<<crate::Driver as Database>::QueryResult>
    where
        E: Executor<'e, Database = crate::Driver>,
        for<'q> <crate::Driver as Database>::Arguments<'q>: IntoArguments<'q, crate::Driver> + Send;

    /// Similar to `update`, but either updates an existing row or inserts a new one if it does not
    /// exist, depending on the primary key's presence and uniqueness.
    async fn upsert<'e, E>(
        &mut self,
        executor: E,
    ) -> Result<<crate::Driver as Database>::QueryResult>
    where
        E: Executor<'e, Database = crate::Driver>,
        for<'q> <crate::Driver as Database>::Arguments<'q>: IntoArguments<'q, crate::Driver> + Send;
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
        for<'q> <crate::Driver as Database>::Arguments<'q>: IntoArguments<'q, crate::Driver> + Send,
    {
        let query = crate::runtime::sql::update::<T>();

        hooks::execute(HookStage::PreBind, &query, HookInput::Row(self)).await?;

        let mut sql = sqlx::query(query.sql());

        for c in query.bindings().columns() {
            sql = self.bind(c, sql).unwrap();
        }

        hooks::execute(HookStage::PreExec, &query, HookInput::None).await?;

        let res = sql
            .persistent(false)
            .execute(executor)
            .await
            .map_err(QueryError::from)
            .map_err(Error::Query);

        hooks::execute(
            hooks::HookStage::PostExec,
            &query,
            QueryResult::Execution(&res).into(),
        )
        .await?;

        res
    }

    async fn upsert<'e, E>(
        &mut self,
        executor: E,
    ) -> Result<<crate::Driver as Database>::QueryResult>
    where
        E: Executor<'e, Database = crate::Driver>,
        for<'q> <crate::Driver as Database>::Arguments<'q>: IntoArguments<'q, crate::Driver> + Send,
    {
        let query = crate::runtime::sql::upsert::<T>();

        hooks::execute(HookStage::PreBind, &query, HookInput::Row(self)).await?;

        let mut sql = sqlx::query(query.sql());

        for c in query.bindings().columns() {
            sql = self.bind(c, sql).unwrap();
        }

        hooks::execute(HookStage::PreExec, &query, HookInput::None).await?;

        let res = sql
            .persistent(false)
            .execute(executor)
            .await
            .map_err(QueryError::from)
            .map_err(Error::Query);

        hooks::execute(
            hooks::HookStage::PostExec,
            &query,
            QueryResult::Execution(&res).into(),
        )
        .await?;

        res
    }
}
