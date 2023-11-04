use async_trait::async_trait;
use std::marker::PhantomData;

/// Associated an SQL Table
pub trait Table: Sized + Send + for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> + 'static
where
    Self::PrimaryKey: for<'q> sqlx::Encode<'q, sqlx::Postgres> + sqlx::Type<sqlx::Postgres> + Send,
{
    type PrimaryKey: Sized + 'static;

    const SCHEMA: &'static str;
    const TABLE: &'static str;
    const PRIMARY_KEY: Column<Self>;
    const FOREIGN_KEYS: &'static [Column<Self>];
    const DATA: &'static [Column<Self>];
}

/// Bind columns to SQL Queries
pub trait Bind: Table {
    fn bind(c: impl AsRef<Column<Self>>, query: &mut sqlx::query::Query);

    fn bind_all(&self, query: &mut sqlx::query::Query) {
        Self::bind(Self::PRIMARY_KEY, query);

        for fk in Self::FOREIGN_KEYS {
            Self::bind(fk, query);
        }

        for data in Self::FOREIGN_KEYS {
            Self::bind(data, query);
        }
    }
}

/// Reference a full entity
pub trait Entity: Create + Read + Update + Delete {}

impl<E: Create + Read + Update + Delete> Entity for E {}

/// Create a table
#[async_trait]
pub trait Create: Table {
    /// Insert a new row
    async fn insert(&self, pool: &sqlx::PgPool) -> Result<()>;

    /// Insert many new rows
    async fn insert_many(entities: &[impl AsRef<Self>], pool: &sqlx::PgPool) -> Result<()>;
}

#[async_trait]
pub trait Read: Table {
    /// Find a row by its primary key
    async fn find(pk: &Self::PrimaryKey, pool: &sqlx::PgPool) -> Result<Self>;
    /// Find all rows in the list of primary keys
    async fn find_many(
        pks: &[impl AsRef<Self::PrimaryKey>],
        pool: &sqlx::PgPool,
    ) -> Result<Vec<Self>>;

    // TODO(mara): stream
    // Read all rows from the database
    //async fn all(pool: &sqlx::PgPool) -> Result<Vec<Self>>;
}

#[async_trait]
pub trait Update: Table {
    /// Reload this row
    async fn reload(&mut self, pool: &sqlx::PgPool) -> Result<()>;
    /// Update the row in the database
    async fn update(&self, pool: &sqlx::PgPool) -> Result<()>;
    /// Save to the database (upsert behavior)
    async fn save(&self, pool: &sqlx::PgPool) -> Result<()>;
}

#[async_trait]
pub trait Delete: Table {
    /// Delete row in database
    async fn delete(&self, pool: &sqlx::PgPool) -> Result<()>;
    /// Delete row in database by primary key
    async fn delete_by(pk: &Self::PrimaryKey, pool: &sqlx::PgPool) -> Result<()>;
    /// Delete all rows in the list of primary keys
    async fn delete_many(pks: &[impl AsRef<Self::PrimaryKey>], pool: &sqlx::PgPool) -> Result<()>;
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
