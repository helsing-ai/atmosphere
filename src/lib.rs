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

pub use atmosphere_core::*;
pub use atmosphere_macros::*;

/// A prelude module for bringing commonly used types into scope
pub mod prelude {
    pub use async_trait::async_trait;
    pub use atmosphere_core::*;
    pub use atmosphere_macros::*;
    pub use sqlx;
}
