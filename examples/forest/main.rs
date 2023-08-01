use atmosphere::prelude::*;
use sqlx::{FromRow, PgPool};

//#[table(name = "forest", schema = "public")]
#[derive(Debug, PartialEq, FromRow, Table)]
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

impl Forest {
    /// Select a forest by its name
    #[query]
    pub async fn by_name(name: &str) -> Result<Self> {
        sql! {
            SELECT * FROM forest
            WHERE name = $name
            ORDER BY name
        }
    }

    // Select all odd trees of this forest
    //#[query]
    //pub async fn trees(&self) -> Result<Tree> {
    //sql! {
    //SELECT * FROM tree
    //WHERE forest_id = $self.id
    //ORDER BY id
    //}
    //}

    /// Burn down the forest
    #[query]
    pub async fn burn(&self) -> Result<()> {
        sql! {
            DELETE FROM tree WHERE forest_id = $self.id
        }
    }
}

//#[table(name = "tree", schema = "public")]
#[derive(Debug, FromRow, Table)]
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
    assert_eq!(&forests, &[grunewald, redwood]);

    //assert_eq!(Forest::by_name("Grunewald", &pool).await?, grunewald);
    //assert_eq!(Forest::by_name("Redwood", &pool).await?, redwood);

    Ok(())
}
