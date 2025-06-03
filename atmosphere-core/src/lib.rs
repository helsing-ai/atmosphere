//! # `🌍 Atmosphere`
//!
//! **A lightweight sql framework for sustainable database reliant systems**
//!
//! ## Overview
//!
//! Atmosphere is a lightweight SQL framework designed for sustainable, database-reliant systems.
//! It leverages Rust's powerful type and macro systems to derive SQL schemas from your rust struct
//! definitions into an advanced trait system.
//!
//! Atmosphere provides a suite of modules and types that abstract and facilitate various aspects
//! of database operations, from query construction and execution to error handling and schema
//! management.
//!
//! ## Key Features
//!
//! - SQL schema derivation from Rust structs.
//! - Advanced trait system for query generation.
//! - Automated database code testing with `atmosphere::testing`
//! - ORM-like CRUD traits.
//! - Code reusability across API layers using generics.
//! - Compile-time introspection for type-safe schema generation.

#![cfg(any(feature = "postgres", feature = "mysql", feature = "sqlite"))]

/// Facilitates binding entities to queries, ensuring type safety and ease of use in query construction.
pub mod bind;
/// Defines high-level database error types, offering a structured approach to error handling.
pub mod error;
/// Implements a hook system, allowing custom logic to be executed at different stages of database
/// interactions.
pub mod hooks;
/// Offers an abstraction layer for building and executing SQL queries, simplifying complex query
/// logic.
pub mod query;
/// Models SQL relationships, providing tools to define and manipulate relationships between
/// database entities.
pub mod rel;
/// Manages the runtime environment for database operations, encompassing execution contexts and
/// configurations.
pub mod runtime;
/// Contains compile-time generated SQL schema traits, enabling a declarative approach to schema
/// definition.
pub mod schema;
/// Provides utilities for automated testing of SQL interactions, ensuring reliability and
/// correctness of database operations.
pub mod testing;

pub use driver::{Driver, Pool};

/// Driver System
///
/// The default driver / feature `any` is activated by default. If a specific driver
/// feature is enabled (`postgres`, `sqlite`, `mysql`) atmosphere will prefer this over
/// the `sqlx::Any` driver.
///
/// If your application makes use of more than one database at the same time, please use the any
/// driver.
pub mod driver {
    #[cfg(any(
        all(feature = "postgres", any(feature = "mysql", feature = "sqlite")),
        all(feature = "mysql", any(feature = "postgres", feature = "sqlite")),
        all(feature = "sqlite", any(feature = "postgres", feature = "mysql")),
    ))]
    compile_error!(
        "only one database driver can be set – please use multiple binaries using different atmosphere features if you need more than one database"
    );

    #[cfg(all(feature = "postgres", not(any(feature = "mysql", feature = "sqlite"))))]
    /// Atmosphere Database Driver
    pub type Driver = sqlx::Postgres;

    #[cfg(all(feature = "postgres", not(any(feature = "mysql", feature = "sqlite"))))]
    /// Atmosphere Database Pool
    pub type Pool = sqlx::PgPool;

    #[cfg(all(feature = "mysql", not(any(feature = "postgres", feature = "sqlite"))))]
    /// Atmosphere Database Driver
    pub type Driver = sqlx::MySql;

    #[cfg(all(feature = "mysql", not(any(feature = "postgres", feature = "sqlite"))))]
    /// Atmosphere Database Pool
    pub type Pool = sqlx::MySqlPool;

    #[cfg(all(feature = "sqlite", not(any(feature = "postgres", feature = "mysql"))))]
    /// Atmosphere Database Driver
    pub type Driver = sqlx::Sqlite;

    #[cfg(all(feature = "sqlite", not(any(feature = "postgres", feature = "mysql"))))]
    /// Atmosphere Database Pool
    pub type Pool = sqlx::SqlitePool;
}

pub use bind::*;
pub use error::*;
pub use schema::*;

#[doc(hidden)]
pub use sqlx;
