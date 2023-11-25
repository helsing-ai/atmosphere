//! SQL Schema
//!
//! This module provides the foundational building blocks for defining the structure and
//! relationships of SQL tables in atmosphere. It includes traits and types that describe table
//! structures, column details, and primary and foreign key relationships. This is essential
//! for representing and manipulating database schema in a type-safe and Rust-idiomatic way.

use sqlx::{Database, Encode, FromRow, Type};

mod create;
mod delete;
mod read;
mod update;

pub use create::Create;
pub use delete::Delete;
pub use read::Read;
pub use update::Update;

pub use self::column::{Column, DataColumn, ForeignKey, PrimaryKey, TimestampColumn};

/// SQL Table Definition
///
/// Represents the definition of a SQL table within the framework, encompassing primary keys,
/// foreign keys, data columns, and timestamp columns. This trait should be implemented by structs
/// that represent database tables, providing metadata and utility functions for table manipulation
/// and query building.
pub trait Table
where
    Self: Sized + Send + for<'r> FromRow<'r, <crate::Driver as Database>::Row> + 'static,
    Self::PrimaryKey: for<'q> Encode<'q, crate::Driver> + Type<crate::Driver> + Send,
{
    /// The type of the primary key for the table.
    type PrimaryKey: Sync + Sized + 'static;

    /// The database schema in which the table resides.
    const SCHEMA: &'static str;
    /// The name of the table.
    const TABLE: &'static str;

    /// The primary key column of the table.
    const PRIMARY_KEY: PrimaryKey<Self>;
    /// An array of foreign key columns.
    const FOREIGN_KEYS: &'static [ForeignKey<Self>];
    /// An array of data columns.
    const DATA_COLUMNS: &'static [DataColumn<Self>];
    /// An array of timestamp columns.
    const TIMESTAMP_COLUMNS: &'static [TimestampColumn<Self>];

    /// Returns a reference to the primary key of the table instance.
    fn pk(&self) -> &Self::PrimaryKey;
}

/// Trait representing an Entity that maps to a database table.
///
/// Entities are table representations that implement CRUD (Create, Read, Update, Delete)
/// operations. This trait is automatically implemented for any type that satisfies the `Create`,
/// `Read`, `Update`, and `Delete` trait requirements, tying together the core functionalities
/// needed for database interaction in the framework.
pub trait Entity: Create + Read + Update + Delete {}

impl<E: Create + Read + Update + Delete> Entity for E {}

/// Column types representing various aspects of table columns.
///
/// These types provide detailed descriptions of table columns, their roles, and their SQL
/// representations. They are used to define the structure of a table and guide query construction
/// and execution within the framework.
pub mod column {
    use crate::Table;
    use std::marker::PhantomData;

    /// An enum that encapsulates different column types of a table.
    #[derive(Copy, Debug, PartialEq, Eq)]
    pub enum Column<T: Table> {
        /// A primary key
        PrimaryKey(&'static PrimaryKey<T>),
        /// A foreign key
        ForeignKey(&'static ForeignKey<T>),
        /// A data column
        Data(&'static DataColumn<T>),
        /// A timestamp column
        Timestamp(&'static TimestampColumn<T>),
    }

    impl<T: Table> Clone for Column<T> {
        fn clone(&self) -> Self {
            match self {
                Self::PrimaryKey(pk) => Self::PrimaryKey(*pk),
                Self::ForeignKey(fk) => Self::ForeignKey(*fk),
                Self::Data(data) => Self::Data(*data),
                Self::Timestamp(ts) => Self::Timestamp(*ts),
            }
        }
    }

    impl<T: Table> Column<T> {
        pub const fn field(&self) -> &'static str {
            match self {
                Self::PrimaryKey(pk) => pk.field,
                Self::ForeignKey(fk) => fk.field,
                Self::Data(data) => data.field,
                Self::Timestamp(ts) => ts.field,
            }
        }

        pub const fn sql(&self) -> &'static str {
            match self {
                Self::PrimaryKey(pk) => pk.sql,
                Self::ForeignKey(fk) => fk.sql,
                Self::Data(data) => data.sql,
                Self::Timestamp(ts) => ts.sql,
            }
        }
    }

    /// Describes the primary key column of a table.
    #[derive(Copy, Debug, PartialEq, Eq)]
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

    impl<T: Table> Clone for PrimaryKey<T> {
        fn clone(&self) -> Self {
            Self {
                field: self.field,
                sql: self.sql,
                table: PhantomData,
            }
        }
    }

    /// Represents a foreign key column, establishing a relationship to another table.
    #[derive(Copy, Debug, PartialEq, Eq)]
    pub struct ForeignKey<T: Table> {
        /// The rust field name of the model
        pub field: &'static str,
        /// The associated sql column name
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

        /// # Safety
        //
        /// We do treat this foreign key as a column of another table. This is not
        /// smart to do - but can become necessary when doing complex joins. This
        /// is memory safe as Self<A> and Self<B> have the exact same memory layout,
        /// we do not store any data (A or B) but only a `PhantomData` instance which
        /// is here transmuted.
        pub const unsafe fn transmute<I: Table>(&'static self) -> &'static ForeignKey<I> {
            std::mem::transmute(self)
        }
    }

    impl<T: Table> Clone for ForeignKey<T> {
        fn clone(&self) -> Self {
            Self {
                field: self.field,
                sql: self.sql,
                table: PhantomData,
            }
        }
    }

    /// Defines a standard data column in the table.
    #[derive(Copy, Debug, PartialEq, Eq)]
    pub struct DataColumn<T: Table> {
        /// The rust field name of the model
        pub field: &'static str,
        /// The associated sql column name
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

        pub const fn as_col(&'static self) -> Column<T> {
            Column::Data(self)
        }
    }

    impl<T: Table> Clone for DataColumn<T> {
        fn clone(&self) -> Self {
            Self {
                field: self.field,
                sql: self.sql,
                table: PhantomData,
            }
        }
    }

    /// The type of a timestamp column
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum TimestampKind {
        Created,
        Updated,
        Deleted,
    }

    /// Specifies a timestamp column, typically used for tracking creation, update, or deletion times.
    #[derive(Copy, Debug, PartialEq, Eq)]
    pub struct TimestampColumn<T: Table> {
        /// The type of this timestamp column
        pub kind: TimestampKind,
        /// The rust field name of the model
        pub field: &'static str,
        /// The associated sql column name
        pub sql: &'static str,
        table: PhantomData<T>,
    }

    impl<T: Table> TimestampColumn<T> {
        pub const fn new(kind: TimestampKind, field: &'static str, sql: &'static str) -> Self {
            Self {
                kind,
                field,
                sql,
                table: PhantomData,
            }
        }
    }

    impl<T: Table> Clone for TimestampColumn<T> {
        fn clone(&self) -> Self {
            Self {
                kind: self.kind,
                field: self.field,
                sql: self.sql,
                table: PhantomData,
            }
        }
    }
}
