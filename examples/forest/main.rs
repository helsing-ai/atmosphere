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
async fn main() -> Result<()> {
    let pool = PgPool::connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .unwrap();

    dbg!(Forest::SCHEMA);
    dbg!(Forest::TABLE);
    dbg!(Forest::PRIMARY_KEY);
    dbg!(Forest::FOREIGN_KEYS);
    dbg!(Forest::DATA);

    let mut insert = atmosphere::runtime::sql::SQL::<Forest, Postgres>::insert();

    println!("{}", insert.sql());

    insert
        .build()
        .bind(1)
        .bind("grunewald")
        .bind("berlin")
        .execute(&pool)
        .await
        .unwrap();

    println!("");
    println!("");
    println!("");
    println!("");
    println!("");
    println!("");
    println!("");

    dbg!(Tree::SCHEMA);
    dbg!(Tree::TABLE);
    dbg!(Tree::PRIMARY_KEY);
    dbg!(Tree::FOREIGN_KEYS);
    dbg!(Tree::DATA);

    Ok(())
}
