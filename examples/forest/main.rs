use atmosphere::prelude::*;
use atmosphere_core::hooks::*;

#[derive(Schema, Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
#[table(schema = "public", name = "forest")]
struct Forest {
    #[sql(pk)]
    id: i32,
    #[sql(unique)]
    name: String,
    #[sql(unique)]
    location: String,
}

impl Hooks for Forest {
    const VALIDATION: &'static [&'static dyn ValidationHook<Self>] = &[];
    const PREPARATION: &'static [&'static dyn PreparationHook<Self>] = &[];
    const INSPECTION: &'static [&'static dyn InspectionHook<Self>] = &[&PrintHook];
    const TRANSPOSITION: &'static [&'static dyn TransposeHook<Self>] = &[];
}

struct PrintHook;

impl<T: Table + Bind> InspectionHook<T> for PrintHook {
    fn apply(&self, ctx: &query::Query<T>) {
        println!(
            "\n\nquerying ({} {:?} {:?}):",
            T::TABLE,
            ctx.op,
            ctx.cardinality
        );
        println!("\n\n{}\n\n", ctx.sql());
    }
}

#[derive(Schema, Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
#[table(schema = "public", name = "tree")]
struct Tree {
    #[sql(pk)]
    id: i32,
    #[sql(fk -> Forest, rename = "forest_id")]
    forest: i32,
}

impl Hooks for Tree {
    const VALIDATION: &'static [&'static dyn ValidationHook<Self>] = &[];
    const PREPARATION: &'static [&'static dyn PreparationHook<Self>] = &[];
    const INSPECTION: &'static [&'static dyn InspectionHook<Self>] = &[&PrintHook];
    const TRANSPOSITION: &'static [&'static dyn TransposeHook<Self>] = &[];
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
