use atmosphere::prelude::*;

#[derive(Schema, Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
#[table(schema = "public", name = "user")]
struct User {
    #[primary_key]
    id: i32,
    name: String,
    #[unique]
    email: String,
}

#[derive(Schema, Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
#[table(schema = "public", name = "post")]
struct Post {
    #[primary_key]
    id: i32,
    #[foreign_key(User)]
    author: i32,
    title: String,
}

#[tokio::main]
async fn main() -> atmosphere::Result<()> {
    let pool = Pool::connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .unwrap();
}
