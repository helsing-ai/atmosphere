use sqlx::{
    database::{HasArguments, HasStatementCache},
    Database, Encode, FromRow, Type,
};
use std::marker::PhantomData;

mod create;
mod delete;
mod read;
mod update;

pub use create::Create;
pub use delete::Delete;
pub use read::Read;
pub use update::Update;

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
