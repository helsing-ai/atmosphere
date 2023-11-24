use sqlx::{Database, Encode, FromRow, Type};

mod create;
mod delete;
mod read;
mod update;

pub use create::Create;
pub use delete::Delete;
pub use read::Read;
pub use update::Update;

pub use self::column::{Column, DataColumn, ForeignKey, MetaColumn, PrimaryKey};

/// SQL Table Definition
pub trait Table
where
    Self: Sized + Send + for<'r> FromRow<'r, <crate::Driver as Database>::Row> + 'static,
    Self::PrimaryKey: for<'q> Encode<'q, crate::Driver> + Type<crate::Driver> + Send,
{
    type PrimaryKey: Sync + Sized + 'static;

    const SCHEMA: &'static str;
    const TABLE: &'static str;

    /// The primary key of this table
    const PRIMARY_KEY: PrimaryKey<Self>;
    /// Columns that are used as a foreign key
    const FOREIGN_KEYS: &'static [ForeignKey<Self>];
    /// Columns that are treated as data
    const DATA_COLUMNS: &'static [DataColumn<Self>];
    /// Columns that are treated as metadata
    const META_COLUMNS: &'static [MetaColumn<Self>];

    fn pk(&self) -> &Self::PrimaryKey;
}

/// A entity is a table that implements [`Create`], [`Read`], [`Update`] & [`Create`]
pub trait Entity: Create + Read + Update + Delete {}

impl<E: Create + Read + Update + Delete> Entity for E {}

pub mod column {
    use crate::Table;
    use std::marker::PhantomData;

    /// Column Variants
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum Column<T: Table> {
        PrimaryKey(&'static PrimaryKey<T>),
        ForeignKey(&'static ForeignKey<T>),
        DataColumn(&'static DataColumn<T>),
        MetaColumn(&'static MetaColumn<T>),
    }

    impl<T: Table> Column<T> {
        pub const fn field(&self) -> &'static str {
            match self {
                Self::PrimaryKey(pk) => pk.field,
                Self::ForeignKey(fk) => fk.field,
                Self::DataColumn(data) => data.field,
                Self::MetaColumn(meta) => meta.field,
            }
        }

        pub const fn sql(&self) -> &'static str {
            match self {
                Self::PrimaryKey(pk) => pk.sql,
                Self::ForeignKey(fk) => fk.sql,
                Self::DataColumn(data) => data.sql,
                Self::MetaColumn(meta) => meta.sql,
            }
        }
    }

    /// Descriptor type of a sql data column
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct PrimaryKey<T: Table> {
        pub field: &'static str,
        pub sql: &'static str,
        table: PhantomData<T>,
    }

    impl<T: Table> PrimaryKey<T> {
        pub const fn new(field: &'static str, sql: &'static str) -> Self {
            Self {
                field,
                sql,
                table: PhantomData,
            }
        }

        pub const fn as_col(&'static self) -> Column<T> {
            Column::PrimaryKey(self)
        }
    }

    /// Descriptor type of a sql foreign key column
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct ForeignKey<T: Table> {
        pub field: &'static str,
        pub sql: &'static str,
        table: PhantomData<T>,
    }

    impl<T: Table> ForeignKey<T> {
        pub const fn new(field: &'static str, sql: &'static str) -> Self {
            Self {
                field,
                sql,
                table: PhantomData,
            }
        }

        pub const fn as_col(&'static self) -> Column<T> {
            Column::ForeignKey(self)
        }

        pub const unsafe fn transmute<I: Table>(&'static self) -> &'static ForeignKey<I> {
            // SAFETY:
            //
            // We do treat this foreign key as a column of another table. This is not
            // smart to do - but can become necessary when doing complex joins. This
            // is memory safe as Self<A> and Self<B> have the exact same memory layout,
            // we do not store any data (A or B) but only a `PhantomData` instance which
            // is here transmuted.
            std::mem::transmute(self)
        }
    }

    /// Descriptor type of a sql data column
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct DataColumn<T: Table> {
        pub field: &'static str,
        pub sql: &'static str,
        table: PhantomData<T>,
    }

    impl<T: Table> DataColumn<T> {
        pub const fn new(field: &'static str, sql: &'static str) -> Self {
            Self {
                field,
                sql,
                table: PhantomData,
            }
        }
    }

    /// Descriptor type of a sql metadata column
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct MetaColumn<T: Table> {
        pub field: &'static str,
        pub sql: &'static str,
        table: PhantomData<T>,
    }

    impl<T: Table> MetaColumn<T> {
        pub const fn new(field: &'static str, sql: &'static str) -> Self {
            Self {
                field,
                sql,
                table: PhantomData,
            }
        }
    }
}
