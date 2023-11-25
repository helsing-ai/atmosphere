use crate::{
    hooks::{self, HookInput, HookStage, Hooks},
    query::{QueryError, QueryResult},
    schema::Table,
    Bind, Error, Result,
};

use async_trait::async_trait;
use sqlx::{database::HasArguments, Executor, IntoArguments};

/// Trait for creating rows in a database.
///
/// This trait provides the functionality to create new rows in a table represented by a struct implementing
/// `Table`, `Bind`, and `Hooks`. It defines an asynchronous method for inserting a new row into the database
/// using a given executor. The trait ensures that all necessary hooks are executed at the appropriate stages
/// of the operation.
#[async_trait]
pub trait Create: Table + Bind + Hooks + Sync + 'static {
    /// Creates a new row in the database. This method builds the SQL insert query,
    /// binds the necessary values, executes the query, and triggers the relevant hooks at different stages
    /// (pre-binding and post-execution).
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

        hooks::execute(HookStage::PreBind, &query, HookInput::Row(&mut self)).await?;

        let mut builder = sqlx::query(query.sql());

        for c in query.bindings().columns() {
            builder = self.bind(c, builder).unwrap();
        }

        let res = builder
            .persistent(false)
            .execute(executor)
            .await
            .map_err(QueryError::from)
            .map_err(Error::Query);

        hooks::execute(
            HookStage::PostExec,
            &query,
            QueryResult::Execution(&res).into(),
        )
        .await?;

        res
    }
}
