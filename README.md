<div align="center">

# `🌍 Atmosphere`

**A lightweight sqlx framework for safe and fast postgres interactions**

[![Hemisphere](https://img.shields.io/badge/hemisphere-open%20source-blueviolet.svg)](https://hemisphere.studio)
[![Hemisphere](https://img.shields.io/badge/postgresql-orm-blue.svg)]()

</div>

## Concept

Atmosphere allows you to derive the sql schema from your rust `struct` definitions into an advanced trait system.

This allows you to:

- Use atmosphere's trait system to generate queries
- Test database code
- Get ORM-like features through CRUD traits building ontop of the above
- Use generics to reuse code across API-layers (e.g. implementing entitiy-generic update http endpoint handler)

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
- [ ] Generate GraphQL + HTTP Server?

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
