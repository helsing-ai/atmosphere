use atmosphere::prelude::*;

use sqlx::types::chrono;
use sqlx::types::chrono::Utc;

#[derive(Schema, Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
#[table(schema = "public", name = "forest")]
struct Forest {
    #[sql(pk)]
    id: i32,
    #[sql(unique)]
    name: String,
    #[sql(unique)]
    location: String,
    #[sql(timestamp = created)]
    created: chrono::DateTime<Utc>,
}

#[derive(Schema, Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
#[table(schema = "public", name = "tree")]
struct Tree {
    #[sql(pk)]
    id: i32,
    #[sql(fk -> Forest, rename = "forest_id")]
    forest: i32,
}

#[tokio::main]
async fn main() -> atmosphere::Result<()> {
    let pool = Pool::connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .unwrap();

    let mut forest = Forest {
        id: 0,
        name: "our".to_owned(),
        location: "forest".to_owned(),
        created: Utc::now(),
    };

    forest.save(&pool).await?;

    for id in 0..5 {
        Tree {
            id,
            forest: forest.id,
        }
        .save(&pool)
        .await?;
    }

    assert_eq!(forest.trees(&pool).await?.len(), 5);

    forest.delete_trees(&pool).await?;

    assert_eq!(forest.trees(&pool).await?.len(), 0);

    Ok(())
}
