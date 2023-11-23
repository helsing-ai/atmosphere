use atmosphere::prelude::*;
use sqlx::PgPool;

#[derive(Schema, Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
#[table(schema = "public", name = "forest")]
struct Forest {
    #[primary_key]
    id: i32,
    name: String,
    location: String,
}

#[derive(Schema, Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
#[table(schema = "public", name = "tree")]
//#[relation(grouped_by = Forest)]
struct Tree {
    #[primary_key]
    id: i32,
    #[foreign_key(Forest)]
    forest_id: i32,
}

#[tokio::main]
async fn main() -> atmosphere::Result<()> {
    //let pool = PgPool::connect(&std::env::var("DATABASE_URL").unwrap())
    //.await
    //.unwrap();

    //Forest {
    //id: 0,
    //name: "test".to_owned(),
    //location: "germany".to_owned(),
    //}
    //.save(&pool)
    //.await?;

    dbg!(Forest::SCHEMA);
    dbg!(Forest::TABLE);
    dbg!(Forest::PRIMARY_KEY);
    dbg!(Forest::FOREIGN_KEYS);
    dbg!(Forest::DATA);

    Ok(())
}
