use atmosphere::{Create, Read, Table as _, table};
use atmosphere_extras::postgis::{Point, Polygon};
use sqlx::{Executor, PgPool};

#[derive(Debug, PartialEq)]
#[table(schema = "public", name = "with_point")]
struct WithPoint {
    #[sql(pk)]
    id: i32,
    point: Point,
}

#[derive(Debug, PartialEq)]
#[table(schema = "public", name = "with_polygon")]
struct WithPolygon {
    #[sql(pk)]
    id: i32,
    polygon: Polygon,
}

#[sqlx::test]
async fn point_roundtrip(pool: PgPool) {
    pool.execute("CREATE EXTENSION postgis").await.unwrap();
    pool.execute(
        r#"
        CREATE TABLE with_point (
          id INT PRIMARY KEY,
          point public.geometry (Point, 4326)
        )
        "#,
    )
    .await
    .unwrap();

    let mut point = WithPoint {
        id: 42,
        point: Point::new(4., 2.),
    };

    let result = point.create(&pool).await.unwrap();

    assert_eq!(result.rows_affected(), 1);

    let from_db = WithPoint::read(&pool, &42).await.unwrap();

    assert_eq!(point, from_db);
}

#[sqlx::test]
async fn polygon_roundtrip(pool: PgPool) {
    pool.execute("CREATE EXTENSION postgis").await.unwrap();
    pool.execute(
        r#"
        CREATE TABLE with_polygon (
          id INT PRIMARY KEY,
          polygon public.geometry (Polygon, 4326)
        )
        "#,
    )
    .await
    .unwrap();

    let mut with_polygon = WithPolygon {
        id: 42,
        polygon: Polygon::from_iter([
            Point::new(0., 0.),
            Point::new(1., 0.),
            Point::new(1., 1.),
            Point::new(0., 1.),
        ]),
    };

    let result = with_polygon.create(&pool).await.unwrap();

    assert_eq!(result.rows_affected(), 1);

    let from_db = WithPolygon::read(&pool, &42).await.unwrap();

    assert_eq!(with_polygon, from_db);
}
