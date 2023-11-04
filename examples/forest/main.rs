#![allow(unused)]

use atmosphere::prelude::*;
use atmosphere_core::Table;
use sqlx::{FromRow, PgPool, Postgres};

#[derive(Schema, Debug)]
#[table(name = "forest", schema = "public")]
struct Forest {
    #[primary_key]
    id: i32,
    name: String,
    location: String,
}

//impl Forest {
//// Select a forest by its name
//#[query(
//r#"
//SELECT {*} FROM {public.forest}
//WHERE name = {name}
//ORDER BY name
//"#
//)]
//pub async fn by_name(name: &str) -> Result<Self>;
//}

#[derive(Schema, Debug)]
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

    let mut forest = Forest {
        id: 1,
        name: "grunewald".to_owned(),
        location: "berlin".to_owned(),
    };

    forest.delete(&pool).await?;
    forest.create(&pool).await?;

    dbg!(Forest::find(&1i32, &pool).await?);

    forest.name = "test".to_owned();
    forest.save(&pool).await?;

    dbg!(Forest::find(&1i32, &pool).await?);

    Ok(())
}
