use atmosphere::prelude::*;
use sqlx::{FromRow, PgPool};

#[derive(Debug, FromRow, Model)]
struct Forest {
    #[id]
    id: i32,
    name: String,
    location: String,
}

impl Forest {
    pub fn new(id: i32, name: impl AsRef<str>, location: impl AsRef<str>) -> Self {
        Self {
            id,
            name: name.as_ref().to_owned(),
            location: location.as_ref().to_owned(),
        }
    }
}

#[derive(Debug, FromRow, Model)]
struct Tree {
    #[id]
    id: i32,
    #[reference(Forest)]
    forest_id: i32,
}

#[tokio::main]
async fn main() -> Result<()> {
    let pool = PgPool::connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .unwrap();

    let grunewald = Forest::new(0, "Grunewald", "Berlin");
    grunewald.delete(&pool).await?;
    grunewald.save(&pool).await?;

    let redwood = Forest::new(1, "Redwood", "USA");
    redwood.delete(&pool).await?;
    redwood.save(&pool).await?;

    dbg!(Forest::all(&pool).await?);

    Ok(())
}
