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

/// Atmosphere Database Driver
pub type Driver = sqlx::Postgres;
/// Atmosphere Database Pool
pub type Pool = sqlx::PgPool;

pub use bind::*;
pub use error::*;
pub use schema::*;
