# Queries

When using Atmosphere, you have two options for writing queries. Once you
derive `Schema` on your entities, it gives you the ability to use querying
traits that Atmosphere comes with. However, you can at any point reach down
and write your queries in raw SQL, the way you would if you used `sqlx`
directly.

## Using Atmosphere Traits

Given the Schema from the section before, here is some examples that show how
Atmosphere creates traits that allow for simple operations on the tables.

```rust
# extern crate atmosphere;
# extern crate sqlx;
# extern crate tokio;
# use atmosphere::prelude::*;
# #[derive(Schema, Debug, PartialEq)]
# #[table(schema = "public", name = "user")]
# struct User {
#     #[sql(pk)]
#     id: i32,
#     name: String,
#     #[sql(unique)]
#     email: String,
# }
# #[derive(Schema, Debug, PartialEq)]
# #[table(schema = "public", name = "post")]
# struct Post {
#     #[sql(pk)]
#     id: i32,
#     #[sql(fk -> User, rename = "author_id")]
#     author: i32,
#     #[sql(unique)]
#     title: String,
# }
# async fn test() -> std::result::Result<(), Box<dyn std::error::Error>> {
let database = std::env::var("DATABASE_URL").unwrap();
let pool = atmosphere::Pool::connect(&database).await?;

let mut user = User {
    id: 0,
    name: "demo".to_owned(),
    email: "some@email.com".to_owned(),
};

user.save(&pool).await?;
user.delete(&pool).await?;
user.create(&pool).await?;

assert_eq!(
    User::read(&pool, &0).await?,
    User::find_by_email(&pool, &"some@email.com".to_string()).await?.unwrap()
);

let mut post = Post {
    id: 0,
    author: 0,
    title: "test".to_owned()
};

post.save(&pool).await?;

Post::find_by_author(&pool, &0).await?;

// Inter-Table Operations

Post { id: 1, author: 0, title: "test1".to_owned() }
    .author(&pool).await?;

user.posts(&pool).await?;
user.delete_posts(&pool).await?;
# Ok(())
# }
# fn main() {}
```

## Using raw SQL

As previously explained, it is always possible to reach down and perform raw SQL
queries on an Atmosphere pool, since it is just an alias for an `sqlx` one.

```rust
# extern crate atmosphere;
# extern crate sqlx;
# extern crate tokio;
# use atmosphere::prelude::*;
# async fn test() -> std::result::Result<(), Box<dyn std::error::Error>> {
let database = std::env::var("DATABASE_URL").unwrap();
let pool = atmosphere::Pool::connect(&database).await?;

sqlx::query("DROP TABLE foo;")
    .execute(&pool).await?;
# Ok(())
# }
# fn main() {}
```
