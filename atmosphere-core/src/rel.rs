//! Provides traits for managing relationships between database entities.
//!
//! This module contains traits and their implementations for handling relationships such as
//! 'RefersTo' and 'ReferredBy'. These traits facilitate operations like resolving and deleting
//! relationships in a database using SQLx.

use async_trait::async_trait;
use sqlx::database::HasArguments;
use sqlx::{Executor, IntoArguments};

use crate::bind::Bind;
use crate::query::{Query, QueryError};
use crate::runtime::sql;
use crate::schema::Table;
use crate::{Error, ForeignKey, Result};

/// Defines a relationship where `Self` refers to `Other`.
///
/// Implements functionality to resolve this relationship, fetching the `Other` entity that `Self`
/// refers to.
#[async_trait]
pub trait RefersTo<Other>
where
    Self: Table + Bind,
    Other: Table + Bind + Unpin + Sync,
{
    const FOREIGN_KEY: ForeignKey<Self>;

    /// Asynchronously resolves and retrieves the `Other` entity that `Self` refers to from the
    /// database.
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

/// Defines a relationship where `Self` is referred to by many `Other`.
///
/// This trait provides methods to resolve these relationships, including fetching all `Other`
/// entities referring to `Self`, resolving by primary key, and deleting all such referring
/// entities.
#[async_trait]
pub trait ReferedBy<Other>
where
    Self: Table + Bind + Unpin + Sync,
    Other: Table + Bind + RefersTo<Self> + Unpin + Sync,
{
    /// Asynchronously fetches all `Other` entities referring to `Self`.
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

    /// Resolves the referring entities based on the primary key of `Self`.
    async fn resolve_by<'e, E>(pk: &Self::PrimaryKey, executor: E) -> Result<Vec<Other>>
    where
        E: Executor<'e, Database = crate::Driver>,
        for<'q> <crate::Driver as HasArguments<'q>>::Arguments:
            IntoArguments<'q, crate::Driver> + Send,
    {
        let Query { builder, .. } = sql::select_by::<Other>(Other::FOREIGN_KEY.as_col());

        sqlx::query_as(builder.sql())
            .bind(pk)
            .persistent(false)
            .fetch_all(executor)
            .await
            .map_err(QueryError::from)
            .map_err(Error::Query)
    }

    /// Deletes all `Other` entities referring to `Self`.
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
