use atmosphere::prelude::*;

#[derive(Debug, Model)]
struct Forest {
    #[id]
    id: i8,
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

    dbg!(Forest::new("Grunewald", "Berlin"));
    Ok(())
}
