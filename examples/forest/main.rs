use atmosphere::prelude::*;
use atmosphere_core::Table;
use sqlx::PgPool;

#[derive(Schema, Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
#[table(name = "forest", schema = "public")]
struct Forest {
    #[primary_key]
    id: i32,
    name: String,
    location: String,
}

#[derive(Schema, Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
#[table(name = "tree", schema = "public")]
#[relation(grouped_by = Forest)]
struct Tree {
    #[primary_key]
    id: i32,
    #[foreign_key(Forest)]
    forest_id: i32,
}

#[tokio::main]
async fn main() -> sqlx::Result<()> {
    let pool = PgPool::connect(&std::env::var("DATABASE_URL").unwrap()).await?;

    Forest {
        id: 0,
        name: "test".to_owned(),
        location: "germany".to_owned(),
    }
    .save(&pool)
    .await?;

    Ok(())
}
