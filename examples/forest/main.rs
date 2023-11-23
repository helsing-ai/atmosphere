use atmosphere::prelude::*;

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
struct Tree {
    #[primary_key]
    id: i32,
    #[foreign_key(Forest)]
    forest_id: i32,
}

#[tokio::main]
async fn main() -> atmosphere::Result<()> {
    let pool = Pool::connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .unwrap();

    Forest {
        id: 0,
        name: "test".to_owned(),
        location: "germany".to_owned(),
    }
    .save(&pool)
    .await?;

    for id in 0..5 {
        Tree { id, forest_id: 0 }.save(&pool).await?;
    }

    dbg!(Forest {
        id: 0,
        name: "test".to_owned(),
        location: "germany".to_owned(),
    }
    .trees(&pool)
    .await?
    .iter()
    .map(|t| t.id)
    .collect::<std::collections::HashSet<i32>>());

    Ok(())
}
