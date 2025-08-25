#[cfg(feature = "serde")]
mod serde {
    use atmosphere_extras::postgis::{Point, Polygon};

    #[test]
    fn serialize_point() {
        let point = Point::from(geo_types::Point::new(4., 2.));

        let serialized = serde_json::to_string(&point).unwrap();
        assert_eq!(serialized, r#"{"x":4.0,"y":2.0}"#);
    }

    #[test]
    fn deserialize_point() {
        let point: Point = serde_json::from_str(r#"{"x":4.0,"y":2.0}"#).unwrap();

        assert_eq!(point, Point::from(geo_types::Point::new(4., 2.)));
    }

    #[test]
    fn serialize_polygon() {
        let exterior = geo_types::LineString::new(vec![
            geo_types::Coord { x: 0., y: 0. },
            geo_types::Coord { x: 1., y: 0. },
            geo_types::Coord { x: 1., y: 1. },
            geo_types::Coord { x: 0., y: 1. },
        ]);
        let polygon = Polygon::from(geo_types::Polygon::new(exterior, Vec::default()));

        let serialized = serde_json::to_string(&polygon).unwrap();
        assert_eq!(
            serialized,
            r#"[{"x":0.0,"y":0.0},{"x":1.0,"y":0.0},{"x":1.0,"y":1.0},{"x":0.0,"y":1.0},{"x":0.0,"y":0.0}]"#
        );
    }

    #[test]
    fn deserialize_polygon() {
        let polygon: Polygon = serde_json::from_str(
            r#"[{"x":0.0,"y":0.0},{"x":1.0,"y":0.0},{"x":1.0,"y":1.0},{"x":0.0,"y":1.0},{"x":0.0,"y":0.0}]"#
        ).unwrap();

        let expected = Polygon::from(geo_types::Polygon::new(
            geo_types::LineString::new(vec![
                geo_types::Coord { x: 0., y: 0. },
                geo_types::Coord { x: 1., y: 0. },
                geo_types::Coord { x: 1., y: 1. },
                geo_types::Coord { x: 0., y: 1. },
            ]),
            Vec::default(),
        ));

        assert_eq!(polygon, expected);
    }
}

#[cfg(feature = "postgis")]
mod postgis {
    use atmosphere::{Create as _, Schema, Table, table};
    use atmosphere_extras::postgis::Point;
    use sqlx::{Executor, PgPool};

    async fn run_setup(pool: &PgPool) {
        pool.execute(
            r#"
            CREATE TABLE has_point (
              id INT PRIMARY KEY,
              point public.geometry (Point, 4326) NOT NULL,
            )
            "#,
        )
        .await
        .unwrap();

        pool.execute(
            r#"
            CREATE TABLE has_polygon (
              id INT PRIMARY KEY,
              point public.geometry (Polygon, 4326)
            )
            "#,
        )
        .await
        .unwrap();
    }

    #[derive(Schema)]
    #[table(schema = "public", name = "has_point")]
    struct Points {
        #[sql(pk)]
        id: i32,
        point: Point,
    }

    #[sqlx::test]
    async fn point_roundtrip(pool: PgPool) {
        run_setup(&pool).await;

        let point = Point::from(geo_types::Point::new(4., 2.));

        point.create(&pool).await.unwrap();
    }
}
