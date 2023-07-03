use atmosphere::prelude::*;
use sqlx::{FromRow, PgPool};

#[derive(Debug, PartialEq, FromRow, Model)]
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

    #[query]
    pub async fn by_name(name: &str) -> Result<Self> {
        select! {
            WHERE name = $name
            ORDER BY name
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

    let grunewald = match Forest::find(&0, &pool).await {
        Ok(forest) => forest,
        Err(_) => {
            let grunewald = Forest::new(0, "Grunewald", "Berlin");
            grunewald.save(&pool).await?;
            grunewald
        }
    };

    let redwood = match Forest::find(&1, &pool).await {
        Ok(forest) => forest,
        Err(_) => {
            let grunewald = Forest::new(0, "Redwood", "USA");
            grunewald.save(&pool).await?;
            grunewald
        }
    };

    let forests = Forest::all(&pool).await?;
    assert_eq!(forests[0], grunewald);
    assert_eq!(forests[1], redwood);

    assert_eq!(Forest::by_name("Grunewald", &pool).await?, grunewald);
    assert_eq!(Forest::by_name("Redwood", &pool).await?, redwood);

    Ok(())
}
