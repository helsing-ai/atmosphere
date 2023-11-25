use crate::{
    hooks::{self, Hooks},
    query::{QueryError, QueryResult},
    schema::Table,
    Bind, Error, Result,
};

use async_trait::async_trait;
use sqlx::{database::HasArguments, Database, Executor, IntoArguments};

/// Trait for deleting rows from a database.
///
/// Provides functionality for deleting rows from a table in the database. Implementors of this trait can delete
/// entities either by their instance or by their primary key. The trait ensures proper execution of hooks at
/// various stages of the delete operation, enhancing flexibility and allowing for custom behavior during the
/// deletion process.
#[async_trait]
pub trait Delete: Table + Bind + Hooks + Send + Sync + Unpin + 'static {
    /// Deletes the row represented by the instance from the database. Builds and executes a delete
    /// query and triggers hooks at appropriate stages (e.g., before binding, before execution,
    /// after execution).
    async fn delete<'e, E>(
        &mut self,
        executor: E,
    ) -> Result<<crate::Driver as Database>::QueryResult>
    where
        E: Executor<'e, Database = crate::Driver>,
        for<'q> <crate::Driver as HasArguments<'q>>::Arguments:
            IntoArguments<'q, crate::Driver> + Send;

    /// Deletes a row from the database based on its primary key. This method is particularly
    /// useful for deleting entities when only the primary key is available.
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

        hooks::execute(
            hooks::HookStage::PreBind,
            &query,
            hooks::HookInput::Row(&mut self),
        )
        .await?;

        let mut sql = sqlx::query(query.sql());

        for c in query.bindings.columns() {
            sql = self.bind(c, sql).unwrap();
        }

        hooks::execute(hooks::HookStage::PreExec, &query, hooks::HookInput::None).await?;

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

        hooks::execute(
            hooks::HookStage::PreBind,
            &query,
            hooks::HookInput::PrimaryKey(pk),
        )
        .await?;

        assert!(query.bindings().columns().len() == 1);
        assert!(query.bindings().columns()[0].field() == Self::PRIMARY_KEY.field);
        assert!(query.bindings().columns()[0].sql() == Self::PRIMARY_KEY.sql);

        hooks::execute(hooks::HookStage::PreExec, &query, hooks::HookInput::None).await?;

        let res = sqlx::query(query.sql())
            .bind(pk)
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
