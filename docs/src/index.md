# Atmosphere

Atmosphere is a lightweight SQL framework designed for sustainable,
database-reliant systems. It leverages Rust's powerful type and macro systems
to derive SQL schemas from your rust struct definitions into an advanced trait
system.

It works by leveraging the [`sqlx`][] crate and the Rust macro system to allow
you to work easily with relational-database mapped entities, while still
enabling low level usage of the underlying `sqlx` concepts. It currently
only supports the Postgres database.

### Sections

**[Getting Started](getting-started/index.md)**

**[Traits](traits/index.md)**

[GitHub]: https://github.com/mara-schulke/atmosphere/tree/main
[`sqlx`]: https://github.com/launchbadge/sqlx
