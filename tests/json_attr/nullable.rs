use atmosphere::prelude::*;
use atmosphere_core::Table;

use super::Data;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
#[table(name = "with_json_nullable", schema = "public")]
struct WithJson {
    #[sql(pk)]
    id: i32,
    #[sql(json)]
    #[sqlx(json(nullable))]
    data: Option<Data>,
}

#[sqlx::test(migrations = "tests/json_attr/migrations")]
async fn create(pool: sqlx::PgPool) {
    atmosphere::testing::create(
        &pool,
        WithJson {
            id: 0,
            data: Some(Data::new("Maria Agnesi")),
        },
    )
    .await;

    atmosphere::testing::create(&pool, WithJson { id: 1, data: None }).await;

    WithJson {
        id: 2,
        data: Some(Data::new("Elizabeth Garrett Anderson")),
    }
    .upsert(&pool)
    .await
    .unwrap();

    WithJson { id: 3, data: None }.upsert(&pool).await.unwrap();
}

#[sqlx::test(migrations = "tests/json_attr/migrations")]
async fn read_some(pool: sqlx::PgPool) {
    atmosphere::testing::read(
        &pool,
        WithJson {
            id: 0,
            data: Some(Data::new("Florence Augusta Merriam Bailey")),
        },
    )
    .await;
}

#[sqlx::test(migrations = "tests/json_attr/migrations")]
async fn read_none(pool: sqlx::PgPool) {
    atmosphere::testing::read(&pool, WithJson { id: 1, data: None }).await;
}

#[sqlx::test(migrations = "tests/json_attr/migrations")]
async fn update(pool: sqlx::PgPool) {
    atmosphere::testing::update(
        &pool,
        WithJson {
            id: 0,
            data: Some(Data::new("Laura Maria Caterina Bassi")),
        },
        vec![
            WithJson { id: 0, data: None },
            WithJson {
                id: 0,
                data: Some(Data::new("Ruth Benerito")),
            },
            WithJson { id: 0, data: None },
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
            data: Some(Data::new("Harriet Brooks")),
        },
    )
    .await;

    atmosphere::testing::delete(&pool, WithJson { id: 1, data: None }).await;
}
