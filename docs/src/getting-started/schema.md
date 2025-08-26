# Define your Schema

To make use of Atmosphere, you must define your schema in a way that Atmosphere
can understand it. To do so, you use Rust structs augmented with the [`table`]
attribute macro and some metadata which tells it how to map it to SQL.

Here is an example of what such a schema might look like if you are storing
users and posts in a database.

```rust
# extern crate atmosphere;
# extern crate sqlx;
use atmosphere::prelude::*;

#[table(schema = "public", name = "user")]
struct User {
    #[sql(pk)]
    id: i32,
    name: String,
    #[sql(unique)]
    email: String,
}

#[table(schema = "public", name = "post")]
struct Post {
    #[sql(pk)]
    id: i32,
    #[sql(fk -> User, rename = "author_id")]
    author: i32,
    #[sql(unique)]
    title: String,
}
# fn main() {
# }
```

## Table properties

Every type you annotate like this corresponds to one table in your Postgres
database. You must set the table and schema name of the entities by setting
the appropriate keys on the `#[table]` annotation.

```rust
# extern crate atmosphere;
# extern crate sqlx;
# use atmosphere::prelude::*;
#[table(schema = "public", name = "users")]
struct User {
    # #[sql(pk)]
    # id: i32,
    // ...
}
# fn main() {
# }
```

## Column properties

Every struct member corresponds to one row of your backing table. Here you can
use the `#[sql]` annotation to add metadata.

[`table`]: https://docs.rs/atmosphere/latest/atmosphere/attr.table.html
