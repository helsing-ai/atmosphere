use crate::{
    query::{Query, QueryError},
    schema::Table,
    Bind, Error, Result,
};

use async_trait::async_trait;
use sqlx::{database::HasArguments, Database, Executor, IntoArguments};

/// Create rows in a [`sqlx::Database`]
#[async_trait]
pub trait Create: Table + Bind + Sync + 'static {
    /// Create a new row
    async fn create<'e, E>(&self, executor: E) -> Result<<Self::Database as Database>::QueryResult>
    where
        E: Executor<'e, Database = Self::Database>,
        for<'q> <Self::Database as HasArguments<'q>>::Arguments:
            IntoArguments<'q, Self::Database> + Send;

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
    ) -> Result<<T::Database as sqlx::Database>::QueryResult>
    where
        E: Executor<'e, Database = Self::Database>,
        for<'q> <Self::Database as HasArguments<'q>>::Arguments:
            IntoArguments<'q, Self::Database> + Send,
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
