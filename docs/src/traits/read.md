# Read

The [`Read`] trait allows you to read entities from rows in your table. Here is
an example of how to create a user, given that you have derived its [`Schema`]:

```rust
# extern crate atmosphere;
# extern crate sqlx;
# extern crate tokio;
# use atmosphere::prelude::*;
#[derive(Schema, Debug, PartialEq)]
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
let user = User::find(&0, &pool).await?;

// refresh user data
# let mut user = user.unwrap();
user.reload(&pool).await?;
# Ok(())
# }
# fn main() {}
```

[`Schema`]: https://docs.rs/atmosphere/latest/atmosphere/derive.Schema.html
[`Read`]: https://docs.rs/atmosphere/latest/atmosphere/trait.Read.html
