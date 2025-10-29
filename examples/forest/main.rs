use atmosphere::prelude::*;

use sqlx::types::chrono;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
#[table(schema = "public", name = "forest")]
struct Forest {
    #[sql(pk)]
    id: i32,
    #[sql(unique)]
    name: String,
    #[sql(unique)]
    location: String,
    #[sql(timestamp = created)]
    created: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
#[table(schema = "public", name = "tree")]
struct Tree {
    #[sql(pk)]
    id: i32,
    #[sql(fk -> Forest, rename = "forest_id")]
    forest: i32,
}

#[tokio::main]
async fn main() -> atmosphere::Result<()> {
    let pool = Pool::connect(":memory:").await.unwrap();

    sqlx::migrate!("examples/forest/migrations")
        .run(&pool)
        .await
        .unwrap();

    let mut forest = Forest {
        id: 0,
        name: "our".to_owned(),
        location: "forest".to_owned(),
        created: chrono::Utc::now(),
    };

    forest.create(&pool).await?;

    for id in 0..5 {
        Tree {
            id,
            forest: forest.id,
        }
        .create(&pool)
        .await?;
    }

    assert_eq!(forest.trees(&pool).await?.len(), 5);

    forest.delete_trees(&pool).await?;

    assert_eq!(forest.trees(&pool).await?.len(), 0);

    Ok(())
}
