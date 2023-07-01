use atmosphere::prelude::*;
use sqlx::FromRow;

#[derive(Debug, FromRow, Model)]
struct Forest {
    #[id]
    id: i32,
    name: String,
    location: String,
}

impl Forest {
    pub fn new(name: impl AsRef<str>, location: impl AsRef<str>) -> Self {
        Self {
            id: 0,
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
    use atmosphere::Model;

    dbg!(
        Forest::SCHEMA,
        Forest::TABLE,
        Forest::ID,
        Forest::REFS,
        Forest::DATA
    );

    dbg!(Tree::SCHEMA, Tree::TABLE, Tree::ID, Tree::REFS, Tree::DATA);

    dbg!(Forest::new("Grunewald", "Berlin"));
    Ok(())
}
