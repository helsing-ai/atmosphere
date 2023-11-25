use crate::{
    hooks::{self, HookInput, HookStage, Hooks},
    query::{QueryError, QueryResult},
    schema::Table,
    Bind, Error, Result,
};

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

        hooks::execute(HookStage::PreBind, &query, HookInput::Row(&mut self)).await?;

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

    async fn save<'e, E>(&mut self, executor: E) -> Result<<crate::Driver as Database>::QueryResult>
    where
        E: Executor<'e, Database = crate::Driver>,
        for<'q> <crate::Driver as HasArguments<'q>>::Arguments:
            IntoArguments<'q, crate::Driver> + Send,
    {
        let query = crate::runtime::sql::upsert::<T>();

        hooks::execute(HookStage::PreBind, &query, HookInput::Row(&mut self)).await?;

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
