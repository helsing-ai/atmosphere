use async_trait::async_trait;
use sqlx::{
    database::{HasArguments, HasStatementCache},
    Database, Encode, Executor, FromRow, IntoArguments, Type,
};
use std::marker::PhantomData;

use crate::{runtime::sql::Query, Bind};

/// SQL Table Definition
pub trait Table
where
    Self: Sized + Send + for<'r> FromRow<'r, <Self::Database as Database>::Row> + 'static,
    Self::PrimaryKey: for<'q> Encode<'q, Self::Database> + Type<Self::Database> + Send,
{
    type Database: Database + HasStatementCache + for<'q> HasArguments<'q>;
    type PrimaryKey: Sync + Sized + 'static;

    const SCHEMA: &'static str;
    const TABLE: &'static str;
    const PRIMARY_KEY: Column<Self>;
    const FOREIGN_KEYS: &'static [Column<Self>];
    const DATA: &'static [Column<Self>];

    fn pk(&self) -> &Self::PrimaryKey;
}

/// A entity is a table that implements [`Create`], [`Read`], [`Update`] & [`Create`]
pub trait Entity: Create + Read + Update + Delete {}

impl<E: Create + Read + Update + Delete> Entity for E {}

/// Create rows in a [`sqlx::Database`]
#[async_trait]
pub trait Create: Table + Bind + Sync + 'static {
    /// Create a new row
    async fn create<'e, E>(
        &self,
        executor: E,
    ) -> sqlx::Result<<Self::Database as Database>::QueryResult>
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
    ) -> sqlx::Result<<T::Database as sqlx::Database>::QueryResult>
    where
        E: Executor<'e, Database = Self::Database>,
        for<'q> <Self::Database as HasArguments<'q>>::Arguments:
            IntoArguments<'q, Self::Database> + Send,
    {
        let Query(builder, bindings) = crate::runtime::sql::insert::<T>();

        let mut query = sqlx::query(builder.sql());

        for c in bindings.columns() {
            query = self.bind(c, query).unwrap();
        }

        query.persistent(false).execute(executor).await
    }
}

/// Read rows from a [`sqlx::Database`]
#[async_trait]
pub trait Read: Table + Bind + Send + Sync + Unpin + 'static {
    /// Find a row by its primary key
    async fn find<'e, E>(pk: &Self::PrimaryKey, executor: E) -> sqlx::Result<Option<Self>>
    where
        E: Executor<'e, Database = Self::Database>,
        for<'q> <Self::Database as HasArguments<'q>>::Arguments:
            IntoArguments<'q, Self::Database> + Send;

    /// Reload from database
    async fn reload<'e, E>(&mut self, executor: E) -> sqlx::Result<()>
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
    async fn find<'e, E>(pk: &Self::PrimaryKey, executor: E) -> sqlx::Result<Option<Self>>
    where
        E: Executor<'e, Database = Self::Database>,
        for<'q> <Self::Database as HasArguments<'q>>::Arguments:
            IntoArguments<'q, Self::Database> + Send,
    {
        let Query(builder, bindings) = crate::runtime::sql::select::<T>();

        dbg!(&bindings);

        assert!(bindings.columns().len() == 1);
        assert!(bindings.columns()[0].name == Self::PRIMARY_KEY.name);

        let query = sqlx::query_as(builder.sql()).bind(pk).persistent(false);

        query.fetch_optional(executor).await
    }

    async fn reload<'e, E>(&mut self, executor: E) -> sqlx::Result<()>
    where
        E: Executor<'e, Database = Self::Database>,
        for<'q> <Self::Database as HasArguments<'q>>::Arguments:
            IntoArguments<'q, Self::Database> + Send,
    {
        let Query(builder, bindings) = crate::runtime::sql::select::<T>();

        let mut query = sqlx::query_as(builder.sql());

        for c in bindings.columns() {
            query = self.bind(c, query).unwrap();
        }

        *self = query.persistent(false).fetch_one(executor).await?;

        Ok(())
    }
}

/// Update rows in a [`sqlx::Database`]
#[async_trait]
pub trait Update: Table + Bind + Send + Sync + Unpin + 'static {
    /// Update the row in the database
    async fn update<'e, E>(
        &self,
        executor: E,
    ) -> sqlx::Result<<Self::Database as Database>::QueryResult>
    where
        E: Executor<'e, Database = Self::Database>,
        for<'q> <Self::Database as HasArguments<'q>>::Arguments:
            IntoArguments<'q, Self::Database> + Send;

    /// Save to the database
    async fn save<'e, E>(
        &self,
        executor: E,
    ) -> sqlx::Result<<Self::Database as Database>::QueryResult>
    where
        E: Executor<'e, Database = Self::Database>,
        for<'q> <Self::Database as HasArguments<'q>>::Arguments:
            IntoArguments<'q, Self::Database> + Send;
}

#[async_trait]
impl<T> Update for T
where
    T: Table + Bind + Send + Sync + Unpin + 'static,
{
    async fn update<'e, E>(
        &self,
        executor: E,
    ) -> sqlx::Result<<Self::Database as Database>::QueryResult>
    where
        E: Executor<'e, Database = Self::Database>,
        for<'q> <Self::Database as HasArguments<'q>>::Arguments:
            IntoArguments<'q, Self::Database> + Send,
    {
        let Query(builder, bindings) = crate::runtime::sql::update::<T>();

        let mut query = sqlx::query(builder.sql());

        for c in bindings.columns() {
            query = self.bind(c, query).unwrap();
        }

        query.persistent(false).execute(executor).await
    }

    async fn save<'e, E>(
        &self,
        executor: E,
    ) -> sqlx::Result<<Self::Database as Database>::QueryResult>
    where
        E: Executor<'e, Database = Self::Database>,
        for<'q> <Self::Database as HasArguments<'q>>::Arguments:
            IntoArguments<'q, Self::Database> + Send,
    {
        let Query(builder, bindings) = crate::runtime::sql::upsert::<T>();

        let mut query = sqlx::query(builder.sql());

        for c in bindings.columns() {
            query = self.bind(c, query).unwrap();
        }

        query.persistent(false).execute(executor).await
    }
}

/// Delete rows from a [`sqlx::Database`]
#[async_trait]
pub trait Delete: Table + Bind + Send + Sync + Unpin + 'static {
    /// Delete row in database
    async fn delete<'e, E>(
        &self,
        executor: E,
    ) -> sqlx::Result<<Self::Database as Database>::QueryResult>
    where
        E: Executor<'e, Database = Self::Database>,
        for<'q> <Self::Database as HasArguments<'q>>::Arguments:
            IntoArguments<'q, Self::Database> + Send;

    /// Delete row in database by primary key
    async fn delete_by<'e, E>(
        pk: &Self::PrimaryKey,
        executor: E,
    ) -> sqlx::Result<<Self::Database as Database>::QueryResult>
    where
        E: Executor<'e, Database = Self::Database>,
        for<'q> <Self::Database as HasArguments<'q>>::Arguments:
            IntoArguments<'q, Self::Database> + Send;

    // Delete all rows in the list of primary keys
    //async fn delete_many<'e, E>(pks: &[impl AsRef<Self::PrimaryKey>], executor: E) -> Result<()>
    //where
    //Self: Bind<sqlx::Postgres> + Sync + 'static,
    //E: sqlx::Executor<'e, Database = sqlx::Postgres>,
    //for<'q> <sqlx::Postgres as sqlx::database::HasArguments<'q>>::Arguments:
    //Send + sqlx::IntoArguments<'q, sqlx::Postgres>;
}

#[async_trait]
impl<T> Delete for T
where
    T: Table + Bind + Send + Sync + Unpin + 'static,
{
    async fn delete<'e, E>(
        &self,
        executor: E,
    ) -> sqlx::Result<<Self::Database as Database>::QueryResult>
    where
        E: Executor<'e, Database = Self::Database>,
        for<'q> <Self::Database as HasArguments<'q>>::Arguments:
            IntoArguments<'q, Self::Database> + Send,
    {
        let Query(builder, bindings) = crate::runtime::sql::delete::<T>();

        let mut query = sqlx::query(builder.sql());

        for c in bindings.columns() {
            query = self.bind(c, query).unwrap();
        }

        query.persistent(false).execute(executor).await
    }

    async fn delete_by<'e, E>(
        pk: &Self::PrimaryKey,
        executor: E,
    ) -> sqlx::Result<<Self::Database as Database>::QueryResult>
    where
        E: Executor<'e, Database = Self::Database>,
        for<'q> <Self::Database as HasArguments<'q>>::Arguments:
            IntoArguments<'q, Self::Database> + Send,
    {
        let Query(builder, bindings) = crate::runtime::sql::delete::<T>();

        assert!(bindings.columns().len() == 1);
        assert!(bindings.columns()[0].name == Self::PRIMARY_KEY.name);

        let query = sqlx::query(builder.sql()).bind(pk).persistent(false);

        query.execute(executor).await
    }
}

/// Descriptor type of a sql column
#[derive(Debug, PartialEq, Eq)]
pub struct Column<T: Table> {
    pub name: &'static str,
    pub ty: ColumnType,
    table: PhantomData<T>,
}

impl<T: Table> Column<T> {
    pub const fn new(name: &'static str, ty: ColumnType) -> Self {
        Self {
            name,
            ty,
            table: PhantomData,
        }
    }
}

/// Different column types in atmosphere
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ColumnType {
    Value,
    PrimaryKey,
    ForeignKey,
}

/// A result type for atmosphere code
pub type Result<T> = std::result::Result<T, ()>;

/// A atmosphere error type
pub enum Error {
    Internal,
}
