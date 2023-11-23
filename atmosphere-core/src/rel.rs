use async_trait::async_trait;
use sqlx::database::HasArguments;
use sqlx::{Executor, IntoArguments};

use crate::bind::Bind;
use crate::query::{Query, QueryError};
use crate::runtime::sql;
use crate::schema::Table;
use crate::{Error, ForeignKey, Result};

/// A relationship where `Self` referrs to `Other`
#[async_trait]
pub trait RefersTo<Other>
where
    Self: Table + Bind,
    Other: Table + Bind + Unpin + Sync,
{
    const FOREIGN_KEY: ForeignKey<Self>;

    async fn resolve<'e, E>(&self, executor: E) -> Result<Other>
    where
        E: Executor<'e, Database = crate::Driver>,
        for<'q> <crate::Driver as HasArguments<'q>>::Arguments:
            IntoArguments<'q, crate::Driver> + Send,
    {
        let Query { builder, .. } = sql::select::<Other>();

        let mut query = sqlx::query_as(builder.sql());

        let fk = Self::FOREIGN_KEY.as_col();
        query = self.bind(&fk, query).unwrap();

        query
            .persistent(false)
            .fetch_one(executor)
            .await
            .map_err(QueryError::from)
            .map_err(Error::Query)
    }
}

/// A relationship where `Self` is referred to by many `Other`
#[async_trait]
pub trait ReferedBy<Other>
where
    Self: Table + Bind + Unpin + Sync,
    Other: Table + Bind + RefersTo<Self> + Unpin + Sync,
{
    async fn resolve<'e, E>(&self, executor: E) -> Result<Vec<Other>>
    where
        E: Executor<'e, Database = crate::Driver>,
        for<'q> <crate::Driver as HasArguments<'q>>::Arguments:
            IntoArguments<'q, crate::Driver> + Send,
    {
        let Query { builder, .. } = sql::select_by::<Other>(Other::FOREIGN_KEY.as_col());

        let mut query = sqlx::query_as(builder.sql());

        let pk = Self::PRIMARY_KEY.as_col();
        query = self.bind(&pk, query).unwrap();

        query
            .persistent(false)
            .fetch_all(executor)
            .await
            .map_err(QueryError::from)
            .map_err(Error::Query)
    }

    async fn delete_all<'e, E>(
        &self,
        executor: E,
    ) -> Result<<crate::Driver as sqlx::Database>::QueryResult>
    where
        E: Executor<'e, Database = crate::Driver>,
        for<'q> <crate::Driver as HasArguments<'q>>::Arguments:
            IntoArguments<'q, crate::Driver> + Send,
    {
        let Query { builder, .. } = sql::delete_by::<Other>(Other::FOREIGN_KEY.as_col());

        let mut query = sqlx::query(builder.sql());

        let pk = Self::PRIMARY_KEY.as_col();
        query = self.bind(&pk, query).unwrap();

        query
            .persistent(false)
            .execute(executor)
            .await
            .map_err(QueryError::from)
            .map_err(Error::Query)
    }
}
