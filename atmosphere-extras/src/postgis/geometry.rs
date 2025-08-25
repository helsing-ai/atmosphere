use sqlx::{Database, Decode, Encode, Postgres, Type};

#[derive(Debug, thiserror::Error)]
pub enum DecodeErr {
    #[error("expected '{expected}', but instead got '{decoded:?}'")]
    WrongType {
        expected: &'static str,
        decoded: geo_types::Geometry,
    },
    #[error("expected a point, but got NULL instead")]
    UnexpectedNull,
}

pub mod point {
    use super::*;

    #[derive(Debug, Clone, PartialEq)]
    pub struct Point(geo_types::Point<f64>);

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
}

mod polygon {
    use sqlx::postgres::PgTypeInfo;

    use super::*;

    #[derive(Debug, Clone, PartialEq)]
    pub struct Polygon(geo_types::Polygon<f64>);

    impl From<geo_types::Polygon<f64>> for Polygon {
        fn from(value: geo_types::Polygon<f64>) -> Self {
            Self(value)
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
}

pub use point::*;
pub use polygon::*;
