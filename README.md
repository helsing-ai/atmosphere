<div align="center">

![Atmosphere](./docs/assets/banner.png)

# `🌍 Atmosphere`

**A lightweight sql framework for sustainable database reliant systems**

[![SQLx](https://img.shields.io/badge/sqlx-framework-blueviolet.svg)]()
[![Crate](https://img.shields.io/crates/v/atmosphere.svg)](https://crates.io/crates/atmosphere)
[![Book](https://img.shields.io/badge/book-latest-0f5225.svg)](https://mara-schulke.github.io/atmosphere)
[![Docs](https://img.shields.io/badge/docs-latest-153f66.svg)](https://docs.rs/atmosphere)

</div>

## Overview

Atmosphere is a lightweight SQL framework designed for sustainable,
database-reliant systems. It leverages Rust's powerful type and macro systems
to derive SQL schemas from your rust struct definitions into an advanced trait
system.

## Key Features

- SQL schema derivation from Rust structs.
- Advanced trait system for query generation.
- Automated database code testing with `atmosphere::testing`
- ORM-like CRUD traits.
- Code reusability across API layers using generics.
- Compile-time introspection for type-safe schema generation.

## Quickstart

```rust
use atmosphere::prelude::*;
use sqlx::PgPool;

#[derive(Schema, Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
#[table(name = "user", schema = "public")]
struct User {
    #[primary_key]
    id: i32,
    name: String,
    email: String,
}

#[tokio::main]
async fn main() -> sqlx::Result<()> {
    let pool = PgPool::connect(&std::env::var("DATABASE_URL").unwrap()).await?;

    User {
        id: 0,
        name: "demo".to_owned(),
        location: "some@email.com".to_owned(),
    }
    .save(&pool)
    .await?;

    Ok(())
}
```

Atmosphere introspects the `User` struct at compile time and generates `const` available type information
about the schema into the `Table` trait:

```rust
impl Table for User {
    const SCHEMA: &str = "public"
    const TABLE: &str = "user"
    const PRIMARY_KEY: Column = Column { name: "id", ty: PrimaryKey, .. };
    const FOREIGN_KEYS: [Column; 0] = [];
    const DATA: [Column; 2] = [Column { name: "name", ty: Value }, Column { name: "email", ty: Value, } ];
}
```

## Roadmap

### Alpha Release
- [x] Advanced SQL Trait System (`Table`, `Column`, `Relation` ..)
- [x] Derive Macro (`Schema`)
- [x] Field Attributes (`#[primary_key]`, `#[foreign_key]` and so on)
- [x] SQL Query Generation
- [x] Automated Integration Testing
- [x] Attribute Macro (`#[table]`)

### Beta Release
- [x] Transaction Support
- [ ] Custom queries
- [ ] Getting Database Agnostic
- [ ] Errors using `miette`
- [ ] Attribute Macro (`#[relation]`)
- [ ] Attribute Macro (`#[query]`)

### Stable Release
- [ ] Stabilize Traits
- [ ] Provide Application Utils
- [ ] Stabilize Query Generation
- [ ] Table Lenses (subsets / views)

### Advanced
- [ ] Postgres Composite Types
- [ ] Support custom types
- [x] Runtime Inspection
- [ ] Generate Graphs
- [ ] `validator` support

### Longterm
- [ ] Generate GraphQL + HTTP Servers?


<!--

## Macros

###### `derive(Schema)`

Builds compile time schema of struct and inserts into global database schema.
This automatically derives the atmosphere base traits for the following common
operations:

**Create**
- `Table::insert(&self)`
- `&[AsRef<Table>]::insert_all(&self)`

**Read**
- `Table::find(id: &Id)`
- `Table::find_all(ids: &[&Id])`

**Update**
- `Table::reload(&mut self)`
- `Table::update(&self)`
- `Table::upsert(&self)`

 **Delete**
- `Table::delete(&mut self)`
- `Table::delete_by(id: &Id)`
- `Table::delete_all_by(ids: &[&Id])`
- `&[AsRef<Table>]::delete_all_by(ids: &[&Id])`

###### `#[query]`
Enables custom queries on the struct

```rust
impl Forest {
    /// Select a forest by its name
    #[query(
        SELECT * FROM ${Forest}
        WHERE name = ${name}
        ORDER BY name
    )]
    pub async fn by_name(name: &str) -> query::Result<Self>;

    /// Select the newest forest
    #[query(
        SELECT * FROM ${Forest}
        ORDER BY created_at DESC
        LIMIT 1
    )]
    pub async fn newest() -> query::Result<Self>;
}
```

---

##### Advanced Macros

###### `#[table(schema = "public", name = <name>, id = (<a>, <b>))]`
configures a table name and schema (`schema.table`)
id optionally tells atmosphere to use combined primary keys

###### `#[relation(grouped_by = Forest)]` and `#[fk(Forest)]`
enable `Tree::by_forest(&forest.id)`

###### `#[relation(groups = Tree)]` and `#[fk(Forest)]`  (on the Tree)
enable `Forest::collect(&self)`

###### `#[relation(links = Forest, as = neighbour)]` and `#[fk(Forest)]`
enable `Tree::neighbour(&self)`

###### `#[virtual(<sql>)]`
marks a virtual column

###### `#[lens(Forest)]`
data lenses on big structs

###### `#[form(Forest)]`
data forms for mutating tables

-->

## Contribution

We welcome contributions! Please see our contribution guidelines for more details.

## License

Atmosphere is licensed under Apache 2.0.
