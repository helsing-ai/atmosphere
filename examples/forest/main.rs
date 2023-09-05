use atmosphere::prelude::*;
use sqlx::{FromRow, PgPool};

#[derive(Schema)]
#[table(name = "forest", schema = "public")]
struct Forest {
    #[primary_key]
    id: i32,
    name: String,
}

impl Forest {
    pub fn new(id: i32, name: &str) -> Self {
        Self {
            id,
            name: name.to_owned(),
        }
    }
}

impl Forest {
    //// Select a forest by its name
    //#[query(
    //r#"
    //SELECT {*} FROM {public.forest}
    //WHERE name = {name}
    //ORDER BY name
    //"#
    //)]
    //pub async fn by_name(name: &str) -> Result<Self>;
}

#[derive(Schema)]
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

    Ok(())
}
