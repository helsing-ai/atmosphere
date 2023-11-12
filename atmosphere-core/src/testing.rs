use crate::Entity;
use sqlx::PgPool;
use std::fmt::Debug;

/// Verify creating of entities
pub async fn create<E>(instance: E, pool: &PgPool)
where
    E: Entity<Database = sqlx::Postgres> + Clone + Debug + Eq + Send,
{
    assert!(
        E::find(&instance.pk(), pool).await.unwrap().is_none(),
        "instance was found before it was created"
    );

    instance.create(pool).await.expect("insertion did not work");

    let retrieved = E::find(&instance.pk(), pool)
        .await
        .unwrap()
        .expect("instance not found after insertion");

    assert_eq!(instance, retrieved);
}

/// Verify read operations
pub async fn read<E>(instance: E, pool: &PgPool)
where
    E: Entity<Database = sqlx::Postgres> + Clone + Debug + Eq + Send,
{
    assert!(
        E::find(&instance.pk(), pool).await.unwrap().is_none(),
        "instance was found after deletion"
    );

    instance.create(pool).await.expect("insertion did not work");

    let retrieved = E::find(&instance.pk(), pool)
        .await
        .unwrap()
        .expect("instance not found after insertion");

    assert_eq!(instance, retrieved);
}

/// Verify update operations
pub async fn update<E>(mut instance: E, updates: Vec<E>, pool: &PgPool)
where
    E: Entity<Database = sqlx::Postgres> + Clone + Debug + Eq + Send,
{
    instance.save(pool).await.expect("insertion did not work");

    for update in updates {
        update
            .update(pool)
            .await
            .expect("updating the instance did not work");

        instance
            .reload(pool)
            .await
            .expect("reloading the instance did not work");

        assert_eq!(instance, update);

        let retrieved = E::find(&instance.pk(), pool)
            .await
            .unwrap()
            .expect("instance not found after update");

        assert_eq!(instance, retrieved);
    }
}

/// Verify delete operations
pub async fn delete<E>(mut instance: E, pool: &PgPool)
where
    E: Entity<Database = sqlx::Postgres> + Clone + Debug + Eq + Send,
{
    instance.create(pool).await.expect("insertion did not work");

    instance.delete(pool).await.expect("deletion did not work");

    instance
        .reload(pool)
        .await
        .expect_err("instance could be reloaded from db after deletion");

    assert!(
        E::find(&instance.pk(), pool).await.unwrap().is_none(),
        "instance was found after deletion"
    );

    instance.create(pool).await.expect("insertion did not work");

    E::delete_by(instance.pk(), pool)
        .await
        .expect("deletion did not work");

    instance
        .reload(pool)
        .await
        .expect_err("instance could be reloaded from db after deletion");
}

// TODO: provide helpers to autogenerate uuids, pks, strings, emails, etc – maybe reexport another
// crate?
