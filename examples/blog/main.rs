use atmosphere::prelude::*;

use sqlx::types::chrono;

#[derive(Schema, Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
#[table(schema = "public", name = "user")]
struct User {
    #[sql(pk)]
    id: i32,
    name: String,
    #[sql(unique)]
    email: String,
}

#[derive(Schema, Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
#[table(schema = "public", name = "post")]
struct Post {
    #[sql(pk)]
    id: i32,
    #[sql(fk -> User)]
    author: i32,
    #[sql(unique)]
    title: String,

    #[sql(timestamp = created)]
    created_at: chrono::DateTime<chrono::Utc>,
    #[sql(timestamp = updated)]
    updated_at: chrono::DateTime<chrono::Utc>,
    #[sql(timestamp = deleted)]
    deleted_at: chrono::DateTime<chrono::Utc>,
}

#[tokio::main]
async fn main() -> atmosphere::Result<()> {
    let pool = Pool::connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .unwrap();

    User {
        id: 0,
        name: "our".to_owned(),
        email: "some@email.com".to_owned(),
    }
    .save(&pool)
    .await?;

    Ok(())
}
