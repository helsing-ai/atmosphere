use sqlx::{Database, Decode, Encode, Postgres, Type};

/// Error related to decoding operations from Postgres via sqlx.
#[derive(Debug, thiserror::Error)]
pub enum DecodeErr {
    /// Indicates that we received a different geometry type from the one we expected.
    #[error("expected '{expected}', but instead got '{decoded:?}'")]
    WrongType {
        expected: &'static str,
        decoded: geo_types::Geometry,
    },
    /// Indicates that we received a `NULL` value instead of a concrete geometry value.
    #[error("expected a non-NULL value, but got NULL instead")]
    UnexpectedNull,
}

pub mod point {
    use super::*;

    /// Wrapper type for PostGIS Point type, which can be used in a table. Provides encoding and
    /// decoding implementations.
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct Point(pub(crate) geo_types::Point<f64>);

    impl Point {
        pub fn new(x: f64, y: f64) -> Self {
            Self(geo_types::Point::new(x, y))
        }
    }

    impl From<geo_types::Point<f64>> for Point {
        fn from(value: geo_types::Point<f64>) -> Self {
            Self(value)
        }
    }

    impl Type<Postgres> for Point {
        fn type_info() -> <Postgres as Database>::TypeInfo {
            sqlx::postgres::PgTypeInfo::with_name("geometry")
        }
    }

    impl<'r> Decode<'r, Postgres> for Point {
        fn decode(
            value: <Postgres as Database>::ValueRef<'r>,
        ) -> Result<Self, sqlx::error::BoxDynError> {
            let decoded = geozero::wkb::Decode::<geo_types::Geometry<f64>>::decode(value)?;

            match decoded.geometry {
                Some(geo_types::Geometry::Point(p)) => Ok(p.into()),
                Some(other) => Err(Box::new(DecodeErr::WrongType {
                    expected: "point",
                    decoded: other,
                })),
                None => Err(Box::new(DecodeErr::UnexpectedNull)),
            }
        }
    }

    impl<'q> Encode<'q, Postgres> for Point {
        fn encode_by_ref(
            &self,
            buf: &mut <Postgres as Database>::ArgumentBuffer<'q>,
        ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
            let geometry = geo_types::Geometry::Point(self.0);
            geozero::wkb::Encode(geometry).encode(buf)
        }
    }

    #[cfg(feature = "serde")]
    mod serde {
        #[derive(serde::Serialize, serde::Deserialize)]
        struct InternalPoint {
            x: f64,
            y: f64,
        }

        impl serde::Serialize for super::Point {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                let point = InternalPoint {
                    x: self.0.x(),
                    y: self.0.y(),
                };

                point.serialize(serializer)
            }
        }

        impl<'de> serde::Deserialize<'de> for super::Point {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                let internal_point = InternalPoint::deserialize(deserializer)?;
                Ok(geo_types::Point::new(internal_point.x, internal_point.y).into())
            }
        }

        #[cfg(test)]
        mod tests {
            use crate::postgis::Point;

            #[test]
            fn serialize_deserialize() {
                let point = Point::new(4., 2.);

                let serialized = serde_json::to_string(&point).unwrap();
                assert_eq!(serialized, r#"{"x":4.0,"y":2.0}"#);

                let deserialized = serde_json::from_str(&serialized).unwrap();
                assert_eq!(point, deserialized);
            }
        }
    }
}

mod polygon {
    use sqlx::postgres::PgTypeInfo;

    use super::*;

    /// A wrapper for the PostGIS `Point` type, providing `Encode` and `Decode` implementations for
    /// database persistence.
    #[derive(Debug, Clone, PartialEq)]
    pub struct Polygon(pub(crate) geo_types::Polygon<f64>);

    impl From<geo_types::Polygon<f64>> for Polygon {
        fn from(value: geo_types::Polygon<f64>) -> Self {
            Self(value)
        }
    }

    impl FromIterator<super::Point> for Polygon {
        fn from_iter<T: IntoIterator<Item = super::Point>>(iter: T) -> Self {
            let exterior = iter.into_iter().map(|point| point.0).collect();
            Self(geo_types::Polygon::new(exterior, Vec::default()))
        }
    }

    impl From<&[super::Point]> for Polygon {
        fn from(points: &[super::Point]) -> Self {
            Self::from_iter(points.iter().copied())
        }
    }

    impl Type<Postgres> for Polygon {
        fn type_info() -> <Postgres as Database>::TypeInfo {
            PgTypeInfo::with_name("geometry")
        }
    }

    impl<'q> Decode<'q, Postgres> for Polygon {
        fn decode(
            value: <Postgres as Database>::ValueRef<'q>,
        ) -> Result<Self, sqlx::error::BoxDynError> {
            let decoded = geozero::wkb::Decode::<geo_types::Geometry<f64>>::decode(value)?;

            match decoded.geometry {
                Some(geo_types::Geometry::Polygon(p)) => Ok(p.into()),
                Some(other) => Err(Box::new(DecodeErr::WrongType {
                    expected: "polygon",
                    decoded: other,
                })),
                None => Err(Box::new(DecodeErr::UnexpectedNull)),
            }
        }
    }

    impl<'r> Encode<'r, Postgres> for Polygon {
        fn encode(
            self,
            buf: &mut <Postgres as Database>::ArgumentBuffer<'r>,
        ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError>
        where
            Self: Sized,
        {
            let geometry = geo_types::Geometry::Polygon(self.0);
            geozero::wkb::Encode(geometry).encode(buf)
        }

        fn encode_by_ref(
            &self,
            buf: &mut <Postgres as Database>::ArgumentBuffer<'r>,
        ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
            self.clone().encode(buf)
        }
    }

    #[cfg(feature = "serde")]
    mod serde {
        #[derive(serde::Serialize, serde::Deserialize)]
        struct InternalPolygon(Vec<super::Point>);

        impl serde::Serialize for super::Polygon {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                let exterior = self.0.exterior();

                let mut points = Vec::with_capacity(exterior.0.len());

                for coord in exterior {
                    let point = geo_types::Point(*coord);
                    points.push(super::Point(point));
                }

                InternalPolygon(points).serialize(serializer)
            }
        }

        impl<'de> serde::Deserialize<'de> for super::Polygon {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                let InternalPolygon(points) = InternalPolygon::deserialize(deserializer)?;
                let coords = points.into_iter().map(|point| point.0.0).collect();
                let exterior = geo_types::LineString::new(coords);
                let polygon = geo_types::Polygon::new(exterior, Vec::default());

                Ok(Self(polygon))
            }
        }

        #[cfg(test)]
        mod tests {
            use crate::postgis::Point;

            use super::super::Polygon;

            #[test]
            fn serialize_deserialize() {
                let polygon = Polygon::from_iter([
                    Point::new(0., 0.),
                    Point::new(1., 0.),
                    Point::new(0., 1.),
                    Point::new(1., 1.),
                ]);

                let serialized = serde_json::to_string(&polygon).unwrap();
                assert_eq!(
                    serialized,
                    r#"[{"x":0.0,"y":0.0},{"x":1.0,"y":0.0},{"x":0.0,"y":1.0},{"x":1.0,"y":1.0},{"x":0.0,"y":0.0}]"#
                );

                let deserialized = serde_json::from_str(&serialized).unwrap();
                assert_eq!(polygon, deserialized);
            }
        }
    }
}

pub use point::*;
pub use polygon::*;
