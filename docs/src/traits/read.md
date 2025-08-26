# Read

The [`Read`] trait allows you to read entities from rows in your table. Here is
an example of how to create a user, given that you have it annotated with
[`table`]:

```rust
# extern crate atmosphere;
# extern crate sqlx;
# extern crate tokio;
# use atmosphere::prelude::*;
#[derive(Debug, PartialEq)]
#[table(schema = "public", name = "user")]
struct User {
    #[sql(pk)]
    id: i32,
    name: String,
    #[sql(unique)]
    email: String,
}

# async fn test() -> std::result::Result<(), Box<dyn std::error::Error>> {
let database = std::env::var("DATABASE_URL").unwrap();
let pool = atmosphere::Pool::connect(&database).await?;

// fetch all users
let users = User::find_all(&pool).await?;

// find user by primary key
let mut user = User::find(&pool, &0).await?;

// refresh user data
user.reload(&pool).await?;
# Ok(())
# }
# fn main() {}
```

[`table`]: https://docs.rs/atmosphere/latest/atmosphere/attr.table.html
[`Read`]: https://docs.rs/atmosphere/latest/atmosphere/trait.Read.html
