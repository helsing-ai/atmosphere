# Create

The [`Create`] trait allows you to create new rows in your tables. Here is an example
of how to create a user, given that you have derived its [`Schema`]:

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

let mut user = User {
    id: 0,
    name: "demo".to_owned(),
    email: "some@email.com".to_owned(),
};

user.create(&pool).await?;
# Ok(())
# }
# fn main() {}
```

[`Schema`]: https://docs.rs/atmosphere/latest/atmosphere/derive.Schema.html
[`Create`]: https://docs.rs/atmosphere/latest/atmosphere/trait.Create.html
