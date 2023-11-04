use async_trait::async_trait;
use sqlx::postgres::PgQueryResult;
use std::marker::PhantomData;

use crate::Bind;

/// Associated an SQL Table
pub trait Table: Sized + Send + for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> + 'static
where
    Self::PrimaryKey: for<'q> sqlx::Encode<'q, sqlx::Postgres> + sqlx::Type<sqlx::Postgres> + Send,
{
    type PrimaryKey: Sync + Sized + 'static;

    const SCHEMA: &'static str;
    const TABLE: &'static str;
    const PRIMARY_KEY: Column<Self>;
    const FOREIGN_KEYS: &'static [Column<Self>];
    const DATA: &'static [Column<Self>];
}

/// Reference a full entity
pub trait Entity: Create + Read + Update + Delete {}

impl<E: Create + Read + Update + Delete> Entity for E {}

/// Create a table
#[async_trait]
pub trait Create: Table {
    /// Create a new row
    async fn create<'e, E>(&self, executor: E) -> sqlx::Result<PgQueryResult>
    where
        Self: Bind<sqlx::Postgres> + Sync + 'static,
        E: sqlx::Executor<'e, Database = sqlx::Postgres>,
        for<'q> <sqlx::Postgres as sqlx::database::HasArguments<'q>>::Arguments:
            Send + sqlx::IntoArguments<'q, sqlx::Postgres>;

    // Create many new rows
    //async fn create_many(entities: &[impl AsRef<Self>], pool: &sqlx::PgPool) -> Result<()> {
    //Self::cre
    //}
}

#[async_trait]
impl<T> Create for T
where
    T: Table,
{
    async fn create<'e, E>(&self, executor: E) -> sqlx::Result<PgQueryResult>
    where
        Self: Bind<sqlx::Postgres> + Sync + 'static,
        E: sqlx::Executor<'e, Database = sqlx::Postgres>,
        for<'q> <sqlx::Postgres as sqlx::database::HasArguments<'q>>::Arguments:
            Send + sqlx::IntoArguments<'q, sqlx::Postgres>,
    {
        let query = crate::runtime::sql::SQL::<T, sqlx::Postgres>::insert();

        self.bind_all(sqlx::query::<sqlx::Postgres>(&query.into_sql()))
            .unwrap()
            .execute(executor)
            .await
    }
}

#[async_trait]
pub trait Read: Table {
    /// Find a row by its primary key
    async fn find<'e, E>(pk: &Self::PrimaryKey, executor: E) -> sqlx::Result<Self>
    where
        Self: Bind<sqlx::Postgres> + Sync + 'static + Unpin,
        E: sqlx::Executor<'e, Database = sqlx::Postgres>,
        for<'q> <sqlx::Postgres as sqlx::database::HasArguments<'q>>::Arguments:
            Send + sqlx::IntoArguments<'q, sqlx::Postgres>;

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
    T: Table,
{
    async fn find<'e, E>(pk: &Self::PrimaryKey, executor: E) -> sqlx::Result<Self>
    where
        Self: Bind<sqlx::Postgres> + Sync + 'static + Unpin,
        E: sqlx::Executor<'e, Database = sqlx::Postgres>,
        for<'q> <sqlx::Postgres as sqlx::database::HasArguments<'q>>::Arguments:
            Send + sqlx::IntoArguments<'q, sqlx::Postgres>,
    {
        let query = crate::runtime::sql::SQL::<T, sqlx::Postgres>::select();

        Ok(sqlx::query_as::<sqlx::Postgres, Self>(&query.into_sql())
            .bind(pk)
            .fetch_one(executor)
            .await
            .unwrap())
    }
}

#[async_trait]
pub trait Update: Table {
    // Reload from database
    //async fn reload<'e, E>(&mut self, executor: E) -> sqlx::Result<()>
    //where
    //Self: Bind<sqlx::Postgres> + Sync + 'static,
    //E: sqlx::Executor<'e, Database = sqlx::Postgres>,
    //for<'q> <sqlx::Postgres as sqlx::database::HasArguments<'q>>::Arguments:
    //Send + sqlx::IntoArguments<'q, sqlx::Postgres>;

    /// Update the row in the database
    async fn update<'e, E>(&mut self, executor: E) -> sqlx::Result<PgQueryResult>
    where
        Self: Bind<sqlx::Postgres> + Sync + 'static,
        E: sqlx::Executor<'e, Database = sqlx::Postgres>,
        for<'q> <sqlx::Postgres as sqlx::database::HasArguments<'q>>::Arguments:
            Send + sqlx::IntoArguments<'q, sqlx::Postgres>;

    /// Save to the database
    async fn save<'e, E>(&mut self, executor: E) -> sqlx::Result<PgQueryResult>
    where
        Self: Bind<sqlx::Postgres> + Sync + 'static,
        E: sqlx::Executor<'e, Database = sqlx::Postgres>,
        for<'q> <sqlx::Postgres as sqlx::database::HasArguments<'q>>::Arguments:
            Send + sqlx::IntoArguments<'q, sqlx::Postgres>;
}

#[async_trait]
impl<T> Update for T
where
    T: Table,
{
    //async fn reload<'e, E>(&self, executor: E) -> Result<PgQueryResult>
    //where
    //Self: Bind<sqlx::Postgres> + Sync + 'static,
    //E: sqlx::Executor<'e, Database = sqlx::Postgres>,
    //for<'q> <sqlx::Postgres as sqlx::database::HasArguments<'q>>::Arguments:
    //Send + sqlx::IntoArguments<'q, sqlx::Postgres>,
    //{
    //let query = crate::runtime::sql::SQL::<T, sqlx::Postgres>::delete();

    //self.bind_all(sqlx::query::<sqlx::Postgres>(&query.into_sql()))?
    //.execute(executor)
    //.await
    //.map_err(|_| ())
    //}

    async fn update<'e, E>(&mut self, executor: E) -> sqlx::Result<PgQueryResult>
    where
        Self: Bind<sqlx::Postgres> + Sync + 'static,
        E: sqlx::Executor<'e, Database = sqlx::Postgres>,
        for<'q> <sqlx::Postgres as sqlx::database::HasArguments<'q>>::Arguments:
            Send + sqlx::IntoArguments<'q, sqlx::Postgres>,
    {
        let query = crate::runtime::sql::SQL::<T, sqlx::Postgres>::update().into_sql();
        let mut query = sqlx::query::<sqlx::Postgres>(&query);
        query = self.bind_all(query).unwrap();
        query.execute(executor).await
    }

    async fn save<'e, E>(&mut self, executor: E) -> sqlx::Result<PgQueryResult>
    where
        Self: Bind<sqlx::Postgres> + Sync + 'static,
        E: sqlx::Executor<'e, Database = sqlx::Postgres>,
        for<'q> <sqlx::Postgres as sqlx::database::HasArguments<'q>>::Arguments:
            Send + sqlx::IntoArguments<'q, sqlx::Postgres>,
    {
        let query = crate::runtime::sql::SQL::<T, sqlx::Postgres>::upsert().into_sql();
        let mut query = sqlx::query::<sqlx::Postgres>(&query);
        query = self.bind_all(query).unwrap();
        query.execute(executor).await
    }
}

#[async_trait]
pub trait Delete: Table {
    /// Delete row in database
    async fn delete<'e, E>(&self, executor: E) -> sqlx::Result<PgQueryResult>
    where
        Self: Bind<sqlx::Postgres> + Sync + 'static,
        E: sqlx::Executor<'e, Database = sqlx::Postgres>,
        for<'q> <sqlx::Postgres as sqlx::database::HasArguments<'q>>::Arguments:
            Send + sqlx::IntoArguments<'q, sqlx::Postgres>;

    /// Delete row in database by primary key
    async fn delete_by<'e, E>(pk: &Self::PrimaryKey, executor: E) -> sqlx::Result<PgQueryResult>
    where
        Self: Bind<sqlx::Postgres> + Sync + 'static,
        E: sqlx::Executor<'e, Database = sqlx::Postgres>,
        for<'q> <sqlx::Postgres as sqlx::database::HasArguments<'q>>::Arguments:
            Send + sqlx::IntoArguments<'q, sqlx::Postgres>;

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
    T: Table,
{
    async fn delete<'e, E>(&self, executor: E) -> sqlx::Result<PgQueryResult>
    where
        Self: Bind<sqlx::Postgres> + Sync + 'static,
        E: sqlx::Executor<'e, Database = sqlx::Postgres>,
        for<'q> <sqlx::Postgres as sqlx::database::HasArguments<'q>>::Arguments:
            Send + sqlx::IntoArguments<'q, sqlx::Postgres>,
    {
        let query = crate::runtime::sql::SQL::<T, sqlx::Postgres>::delete();

        self.bind_all(sqlx::query::<sqlx::Postgres>(&query.into_sql()))
            .unwrap()
            .execute(executor)
            .await
    }

    async fn delete_by<'e, E>(pk: &Self::PrimaryKey, executor: E) -> sqlx::Result<PgQueryResult>
    where
        Self: Bind<sqlx::Postgres> + Sync + 'static,
        E: sqlx::Executor<'e, Database = sqlx::Postgres>,
        for<'q> <sqlx::Postgres as sqlx::database::HasArguments<'q>>::Arguments:
            Send + sqlx::IntoArguments<'q, sqlx::Postgres>,
    {
        let query = crate::runtime::sql::SQL::<T, sqlx::Postgres>::delete();

        sqlx::query::<sqlx::Postgres>(&query.into_sql())
            .bind(pk)
            .execute(executor)
            .await
    }
}

#[derive(Debug)]
pub struct Column<T: Table> {
    pub name: &'static str,
    pub ty: ColType,
    table: PhantomData<T>,
}

impl<T: Table> Column<T> {
    pub const fn new(name: &'static str, ty: ColType) -> Self {
        Self {
            name,
            ty,
            table: PhantomData,
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ColType {
    Value,
    PrimaryKey,
    ForeignKey,
}

pub type Result<T> = std::result::Result<T, ()>;
