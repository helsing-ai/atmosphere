//! # `🌍 Atmosphere Extras`
//!
//! Implementations for additional database types, such as types from PostGIS plugin for Postgres.

#![cfg(any(feature = "postgres"))]

#[cfg(feature = "postgis")]
pub mod postgis;
