use crate::{
    query::{Query, QueryError},
    schema::Table,
    Bind, Error, Result,
};

use async_trait::async_trait;
use sqlx::{database::HasArguments, Executor, IntoArguments};

/// Create rows in a [`sqlx::Database`]
#[async_trait]
pub trait Create: Table + Bind + Sync + 'static {
    /// Create a new row
    async fn create<'e, E>(
        &self,
        executor: E,
    ) -> Result<<crate::Driver as sqlx::Database>::QueryResult>
    where
        E: Executor<'e, Database = crate::Driver>,
        for<'q> <crate::Driver as HasArguments<'q>>::Arguments:
            IntoArguments<'q, crate::Driver> + Send;

    // Create many new rows
    //async fn create_many(entities: &[impl AsRef<Self>], pool: &sqlx::PgPool) -> Result<()> {
    //Self::cre
    //}
}

#[async_trait]
impl<T> Create for T
where
    T: Table + Bind + Sync + 'static,
{
    async fn create<'e, E>(
        &self,
        executor: E,
    ) -> Result<<crate::Driver as sqlx::Database>::QueryResult>
    where
        E: Executor<'e, Database = crate::Driver>,
        for<'q> <crate::Driver as HasArguments<'q>>::Arguments:
            IntoArguments<'q, crate::Driver> + Send,
    {
        let Query {
            builder, bindings, ..
        } = crate::runtime::sql::insert::<T>();

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
}
