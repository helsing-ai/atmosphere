use atmosphere::prelude::*;
use atmosphere_core::Table;

#[derive(Schema, Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
#[table(name = "forest", schema = "public")]
struct Forest {
    #[primary_key]
    id: i32,
    name: String,
    location: String,
}

#[derive(Schema, Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
#[table(name = "tree", schema = "public")]
struct Tree {
    #[primary_key]
    id: i32,
    #[foreign_key(Forest)]
    forest_id: i32,
}

#[sqlx::test(migrations = "tests/db/migrations")]
async fn create(pool: sqlx::PgPool) {
    atmosphere::testing::create(
        Forest {
            id: 0,
            name: "grunewald".to_owned(),
            location: "berlin".to_owned(),
        },
        &pool,
    )
    .await;
}

#[sqlx::test(migrations = "tests/db/migrations")]
async fn read(pool: sqlx::PgPool) {
    atmosphere::testing::read(
        Forest {
            id: 0,
            name: "grunewald".to_owned(),
            location: "berlin".to_owned(),
        },
        &pool,
    )
    .await;
}

#[sqlx::test(migrations = "tests/db/migrations")]
async fn update(pool: sqlx::PgPool) {
    atmosphere::testing::update(
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
        &pool,
    )
    .await;
}

#[sqlx::test(migrations = "tests/db/migrations")]
async fn delete(pool: sqlx::PgPool) {
    atmosphere::testing::delete(
        Forest {
            id: 0,
            name: "grunewald".to_owned(),
            location: "berlin".to_owned(),
        },
        &pool,
    )
    .await;
}
