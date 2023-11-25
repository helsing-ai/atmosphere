use atmosphere::hooks::*;
use atmosphere::prelude::*;

use atmosphere::query::Query;

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

#[derive(Schema, Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
#[table(schema = "public", name = "tree")]
struct Tree {
    #[sql(pk)]
    id: i32,
    #[sql(fk -> Forest, rename = "forest_id")]
    forest: i32,
}

mod logging {
    use super::*;

    struct PrintHook;

    #[async_trait]
    impl<T: Table + Bind + Sync> Hook<T> for PrintHook {
        fn stage(&self) -> HookStage {
            HookStage::PreExec
        }

        async fn apply(&self, ctx: &Query<T>, _: &HookInput<'_, T>) -> Result<()> {
            println!(
                "atmosphere::logs::{} => {:?} {:?}",
                T::TABLE,
                ctx.op,
                ctx.cardinality,
            );

            Ok(())
        }
    }

    impl Hooks for Forest {
        const HOOKS: &'static [&'static dyn Hook<Self>] = &[&PrintHook];
    }

    impl Hooks for Tree {
        const HOOKS: &'static [&'static dyn Hook<Self>] = &[&PrintHook];
    }
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
