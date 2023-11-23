/// Bind entities to queries
pub mod bind;
/// High level database error types
pub mod error;
/// Hook system
pub mod hooks;
/// Query abstraction
pub mod query;
/// Abstraction to model sql relationships
pub mod rel;
/// Runtime environment
pub mod runtime;
/// Compile time generated SQL schema traits
pub mod schema;
/// Automated testing of SQL interactions
pub mod testing;

/// Atmosphere Database Driver
pub type Driver = sqlx::Postgres;
/// Atmosphere Database Pool
pub type Pool = sqlx::PgPool;

pub use bind::*;
pub use error::*;
pub use schema::*;
