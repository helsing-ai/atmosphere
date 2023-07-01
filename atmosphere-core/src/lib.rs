use async_trait::async_trait;
use std::marker::PhantomData;

pub trait Model: Sized + Send + 'static
where
    Self::Id: for<'q> sqlx::Encode<'q, sqlx::Postgres> + sqlx::Type<sqlx::Postgres> + Send,
{
    type Id: Sized + 'static;

    const SCHEMA: &'static str;
    const TABLE: &'static str;
    const ID: Column<Self>;
    const REFS: &'static [Column<Self>];
    const DATA: &'static [Column<Self>];
}

#[async_trait]
pub trait Read: Model {
    async fn find(id: &Self::Id, pool: &sqlx::PgPool) -> Result<Self>;
    async fn all(pool: &sqlx::PgPool) -> Result<Vec<Self>>;
    // async fn select(filter: Vec<Filter<Self>>) -> Result<Vec<Self>>;
}

#[async_trait]
pub trait Write: Model {
    async fn save(&self) -> Result<()>;
    async fn update(&self) -> Result<()>;
    async fn delete(&self) -> Result<()>;
}

#[derive(Debug)]
pub struct Column<M: Model> {
    pub name: &'static str,
    pub data_type: DataType,
    pub col_type: ColType,
    marker: PhantomData<M>,
}

impl<M: Model> Column<M> {
    pub const fn new(name: &'static str, data_type: DataType, col_type: ColType) -> Self {
        Self {
            name,
            data_type,
            col_type,
            marker: PhantomData,
        }
    }
}

//#[derive(Debug)]
//pub struct Reference<A: Model, B: Model> {
//pub column: Column<A>,
//marker: PhantomData<B>
//}

//impl<A: Model, B: Model> for Reference<A, B> {
//pub const fn new(col: Column<A>) -> Self {
//Self {
//column,
//marker: PhantomData,
//}
//}
//}

/// All possible types for postgres
#[derive(Debug)]
pub enum DataType {
    Unknown,
    Text,
    Number,
}

#[derive(Debug)]
pub enum ColType {
    Value,
    PrimaryKey,
    ForeignKey,
}

//mod query {
//use sqlx::Postgres;

//use crate::{Column, Model};

//pub(crate) trait Query<M: Model> {
//fn build(&self) -> String;
//}

//pub struct Select<M: Model> {
//filter: Vec<Filter<M>>,
//}

//impl<M: Model> Select<M> {
//pub(crate) fn new() -> Self {
//Self { filter: vec![] }
//}

//pub(crate) fn filtered(filter: Filter<M>) -> Self {
//Self {
//filter: vec![filter],
//}
//}
//}

//impl<M: Model> Query<M> for Select<M> {
//fn build(&self) -> String {
//let mut builder = sqlx::QueryBuilder::<Postgres>::new(format!(
//"SELECT * FROM {}.{}",
//M::SCHEMA,
//M::TABLE
//));

//if !self.filter.is_empty() {
//builder.push("WHERE");
//}

//for filter in self.filter {
//builder.push(filter.column.name);
//builder.push(filter.op);
//}

//builder.into_sql()
//}
//}

//pub struct Filter<M: Model> {
//pub column: Column<M>,
//pub op: FilterOperation,
//}

//pub enum FilterOperation {
////Greater(Box<dyn sqlx::Encode<'static, Postgres>>),
////GreaterOrEqual(Box<dyn sqlx::Encode<'static, Postgres>>),
////Equal(Box<dyn sqlx::Encode<'static, Postgres>>),
////Less(Box<dyn sqlx::Encode<'static, Postgres>>),
////LessThan(Box<dyn sqlx::Encode<'static, Postgres>>),
//NotNull,
//IsNull,
//}
//}

pub type Result<T> = std::result::Result<T, ()>;

#[cfg(test)]
mod tests {
    use sqlx::PgPool;

    use super::*;

    #[allow(unused)]
    struct Foo {
        id: i8,
    }

    impl Model for Foo {
        type Key = i8;

        const SCHEMA: &'static str = "public";
        const TABLE: &'static str = "foo";

        const KEY: Column<Self> = Column::new("id", DataType::Number, ColType::PrimaryKey);
        const REFS: &'static [Column<Self>] = &[];
        const DATA: &'static [Column<Self>] = &[];
    }

    #[async_trait]
    impl Read for Foo {
        async fn by(id: &i8) -> Result<Self> {
            Err(())
        }

        async fn all() -> Result<Vec<Self>> {
            Err(())
        }
    }

    #[sqlx::test]
    async fn test(pool: PgPool) {}
}
