use atmosphere::prelude::*;

#[derive(Model)]
#[atmosphere(table = "forest")]
struct Forest {
    #[id]
    id: i8,
    name: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    Ok(())
}
