use atmosphere::prelude::*;
use atmosphere_core::Table;

#[derive(Schema, Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
#[table(name = "forest", schema = "public")]
struct Forest {
    #[sql(pk)]
    id: i32,
    name: String,
    location: String,
}

#[derive(Schema, Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
#[table(name = "tree", schema = "public")]
struct Tree {
    #[sql(pk)]
    id: i32,
    #[sql(fk -> Forest, rename = "forest_id")]
    forest: i32,
}

#[sqlx::test(migrations = "tests/db/migrations")]
async fn create(pool: sqlx::PgPool) {
    atmosphere::testing::create(
        &pool,
        Forest {
            id: 0,
            name: "grunewald".to_owned(),
            location: "berlin".to_owned(),
        },
    )
    .await;

    Forest {
        id: 99,
        name: "place".to_owned(),
        location: "holder".to_owned(),
    }
    .upsert(&pool)
    .await
    .unwrap();

    atmosphere::testing::create(&pool, Tree { id: 0, forest: 99 }).await;
}

#[sqlx::test(migrations = "tests/db/migrations")]
async fn read(pool: sqlx::PgPool) {
    atmosphere::testing::read(
        &pool,
        Forest {
            id: 0,
            name: "grunewald".to_owned(),
            location: "berlin".to_owned(),
        },
    )
    .await;

    Forest {
        id: 99,
        name: "place".to_owned(),
        location: "holder".to_owned(),
    }
    .upsert(&pool)
    .await
    .unwrap();

    atmosphere::testing::read(&pool, Tree { id: 0, forest: 99 }).await;
}

#[sqlx::test(migrations = "tests/db/migrations")]
async fn update(pool: sqlx::PgPool) {
    atmosphere::testing::update(
        &pool,
        Forest {
            id: 0,
            name: "grunewald".to_owned(),
            location: "berlin".to_owned(),
        },
        vec![
            Forest {
                id: 0,
                name: "gruneeeeeeeewald".to_owned(),
                location: "berlin".to_owned(),
            },
            Forest {
                id: 0,
                name: "grunewald".to_owned(),
                location: "berlin, germany".to_owned(),
            },
            Forest {
                id: 0,
                name: "englischer garten".to_owned(),
                location: "münchen".to_owned(),
            },
        ],
    )
    .await;

    Forest {
        id: 99,
        name: "place".to_owned(),
        location: "holder".to_owned(),
    }
    .upsert(&pool)
    .await
    .unwrap();

    Forest {
        id: 100,
        name: "place".to_owned(),
        location: "holder".to_owned(),
    }
    .upsert(&pool)
    .await
    .unwrap();

    atmosphere::testing::update(
        &pool,
        Tree { id: 0, forest: 99 },
        vec![Tree { id: 0, forest: 100 }, Tree { id: 0, forest: 99 }],
    )
    .await;
}

#[sqlx::test(migrations = "tests/db/migrations")]
async fn delete(pool: sqlx::PgPool) {
    atmosphere::testing::delete(
        &pool,
        Forest {
            id: 0,
            name: "grunewald".to_owned(),
            location: "berlin".to_owned(),
        },
    )
    .await;

    Forest {
        id: 99,
        name: "place".to_owned(),
        location: "holder".to_owned(),
    }
    .upsert(&pool)
    .await
    .unwrap();

    atmosphere::testing::delete(&pool, Tree { id: 0, forest: 99 }).await;
}
