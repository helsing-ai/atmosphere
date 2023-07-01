use atmosphere::prelude::*;

#[derive(Debug, Model)]
#[atmosphere(table = "forest")]
struct Forest {
    #[id]
    id: i8,
    name: String,
}

impl Forest {
    pub fn new(name: impl AsRef<str>) -> Self {
        Self {
            id: 0,
            name: name.as_ref().to_owned(),
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    use atmosphere::Model;

    dbg!(
        Forest::SCHEMA,
        Forest::TABLE,
        Forest::KEY,
        Forest::REFS,
        Forest::DATA
    );

    dbg!(Forest::new("Grunewald"));
    Ok(())
}
