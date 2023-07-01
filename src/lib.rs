use std::marker::PhantomData;

use async_trait::async_trait;

pub trait Model: Sized + 'static {
    type Key: Sized + 'static;

    const SCHEMA: &'static str;
    const TABLE: &'static str;
    const KEY: Column<Self>;
    const REFS: &'static [Column<Self>];
    const DATA: &'static [Column<Self>];
}

#[async_trait]
pub trait Read: Model {
    async fn by(key: &Self::Key) -> Result<Self>;
    async fn all() -> Result<Vec<Self>>;
}

#[async_trait]
pub trait Write: Model {
    async fn save(&self) -> Result<()>;
    async fn update() -> Result<()>;
    async fn delete() -> Result<()>;
}

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

/// All possible types for postgres
pub enum DataType {
    Text,
    Number,
}

pub enum ColType {
    Value,
    PrimaryKey,
    ForeignKey,
}

type Result<T> = std::result::Result<T, ()>;

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(unused)]
    struct Foo {
        id: u8,
    }

    impl Model for Foo {
        type Key = u8;

        const SCHEMA: &'static str = "public";
        const TABLE: &'static str = "foo";

        const KEY: Column<Self> = Column::new("id", DataType::Number, ColType::PrimaryKey);
        const REFS: &'static [Column<Self>] = &[];
        const DATA: &'static [Column<Self>] = &[];
    }
}
