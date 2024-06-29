<div align="center">

![Atmosphere](./docs/assets/banner.png)

# `🌍 Atmosphere`

**A lightweight sql framework for sustainable database reliant systems**

[![SQLx](https://img.shields.io/badge/sqlx-framework-blueviolet.svg)](https://github.com/launchbadge/sqlx)
[![Crate](https://img.shields.io/crates/v/atmosphere.svg)](https://crates.io/crates/atmosphere)
[![Book](https://img.shields.io/badge/book-latest-0f5225.svg)](https://helsing-ai.github.io/atmosphere)
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

#[derive(Schema)]
#[table(schema = "public", name = "user")]
struct User {
    #[sql(pk)]
    id: i32,
    name: String,
    #[sql(unique)]
    email: String,
}

#[derive(Schema)]
#[table(schema = "public", name = "post")]
struct Post {
    #[sql(pk)]
    id: i32,
    #[sql(fk -> User, rename = "author_id")]
    author: i32,
    #[sql(unique)]
    title: String,
}

#[tokio::main]
async fn main() -> sqlx::Result<()> {
    let pool = atmosphere::Pool::connect(&std::env::var("DATABASE_URL").unwrap()).await?;

    // CRUD operations

    let user = User { id: 0, name: "demo".to_owned(), email: "some@email.com".to_owned(), };

    user.save(&pool).await?;
    user.delete(&pool).await?;
    user.create(&pool).await?;

    // Field Queries

    assert_eq!(
        User::read(&pool, &0).await?,
        User::find_by_email(&pool, "some@email.com").await?.unwrap()
    );

    // Relationships

    Post { id: 0, author: 0, title: "test".to_owned() }
        .save(&pool)
        .await?;

    Post::find_by_author(&pool, &0).await?;
    Post::delete_by_author(&pool, &0).await?;

    // Inter-Table Operations

    Post { id: 1, author: 0, title: "test1".to_owned() }
        .author(&pool).await?;

    user.posts(&pool).await?;
    user.delete_posts(&pool).await?;

    Ok(())
}
```

Atmosphere introspects the `User` and `Post` structs at compile time and
generates `const` available type information about the schema into the `Table`
trait.

## Roadmap

### Alpha Release
- [x] Advanced SQL Trait System (`Table`, `Column`, `Relation` ..)
- [x] Derive Macro (`Schema`)
- [x] SQL Field Attributes (`#[sql(pk)]`, `#[sql(fk -> Model)]` and so on)
- [x] SQL Query Generation
- [x] Automated Integration Testing
- [x] Attribute Macro (`#[table]`)

### Beta Release
- [x] Transaction Support
- [x] Getting Database Agnostic
- [x] Hook into query execution using `atmosphere::hooks`
- [x] Errors using `miette`
- [ ] Combined Primary and Foreign Keys

### Stable Release
- [x] Postgres Composite Types
- [x] Support Custom Types
- [x] Runtime Inspection
- [x] Provide Application Utils
- [ ] Stabilize Traits
- [ ] Stabilize Query Generation
- [ ] Table Lenses (subsets / views)
- [ ] `validator` support
- [ ] Auto Timestamping

### Advanced
- [ ] Virtual Columns using (`#[virtual = "<sql>"]`)
- [ ] Soft Delete Support
- [ ] Attribute Macro (`#[query]`)
- [ ] Custom queries

### Longterm
- [ ] Generate GraphQL + HTTP Servers?
- [ ] Generate Graphs


## Functionalities

Given a `struct Model` that derives its atmosphere schema using
`#[derive(Schema)]` and `#[table]`:

```rust
use atmosphere::prelude::*;

#[derive(Schema)]
#[table(schema = "public", name = "model")]
struct Model {
    #[sql(pk)]
    id: i32,
    a: String,
    #[sql(unique)]
    b: String,
}
```

Atmosphere is able to derive and generate the following queries:

### CRUD

#### `atmosphere::Create`

- `Model::create`

#### `atmosphere::Read`

- `Model::read`: read a `Model` by its primary key, returning a `Model`.
- `Model::find`: find a `Model` by its primary key, returning an `Option<Model>`.
- `Model::read_all`: read all `Model`s, returning a `Vec<Model>`.
- `Model::reload`

#### `atmosphere::Update`

- `Model::update`
- `Model::upsert`

#### `atmosphere::Delete`

- `Model::delete`
- `Model::delete_by`

### Field Queries

Each struct field that is marked with `#[sql(unique)]` becomes queryable.

In the above example `b` was marked as unique so atmosphere implements:

- `Model::find_by_b`: find a `Model` by its `b` field, returning an `Option<Model>`.
- `Model::delete_by_b`: delete a `Model` by its `b` field.

### Relationships & Inter-Table Queries

Given that a model contains fields are marked as a foreign key / point to
another `atmosphere::Table` atmosphere – for example:

```rust
#[derive(Schema)]
#[table(schema = "public", name = "submodel")]
struct Submodel {
    #[sql(pk)]
    id: i32,
    #[sql(fk -> Model)]
    super: i32,
}
```

Atmosphere is able to generate utility queries to move across `Table` boundaries:

- `Model::submodels`
- `Model::delete_submodels`
- `Submodel::model`
- `Submodel::find_by_model`
- `Submodel::delete_by_model`

> Note that the function names contain `model` and `submodel` – they are derived from
> the respective struct names.

## Contribution

We welcome contributions! Please see [our contribution guidelines](CONTRIBUTING.md) for more details.

## License

Atmosphere is licensed under Apache 2.0.
