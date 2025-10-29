use atmosphere::prelude::*;
use atmosphere_core::Table;

use super::Data;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
#[table(name = "with_json_nullable", schema = "public")]
struct WithJson {
    #[sql(pk)]
    id: i32,
    #[sql(json)]
    #[sqlx(json)]
    data: Data,
}

#[sqlx::test(migrations = "tests/json_attr/migrations")]
async fn create(pool: sqlx::PgPool) {
    atmosphere::testing::create(
        &pool,
        WithJson {
            id: 0,
            data: Data::new("Maria Agnesi"),
        },
    )
    .await;

    WithJson {
        id: 1,
        data: Data::new("Elizabeth Garrett Anderson"),
    }
    .upsert(&pool)
    .await
    .unwrap();
}

#[sqlx::test(migrations = "tests/json_attr/migrations")]
async fn read(pool: sqlx::PgPool) {
    atmosphere::testing::read(
        &pool,
        WithJson {
            id: 0,
            data: Data::new("Florence Augusta Merriam Bailey"),
        },
    )
    .await;
}

#[sqlx::test(migrations = "tests/json_attr/migrations")]
async fn update(pool: sqlx::PgPool) {
    atmosphere::testing::update(
        &pool,
        WithJson {
            id: 0,
            data: Data::new("Laura Maria Caterina Bassi"),
        },
        vec![
            WithJson {
                id: 0,
                data: Data::new("Ruth Benerito"),
            },
            WithJson {
                id: 0,
                data: Data::new("Marie Curie"),
            },
        ],
    )
    .await;
}

#[sqlx::test(migrations = "tests/json_attr/migrations")]
async fn delete(pool: sqlx::PgPool) {
    atmosphere::testing::delete(
        &pool,
        WithJson {
            id: 0,
            data: Data::new("Harriet Brooks"),
        },
    )
    .await;
}
