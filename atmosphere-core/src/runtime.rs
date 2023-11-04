use std::{collections::HashSet, hash::Hash};

use crate::{ColType, DataType};

lazy_static::lazy_static! {
    pub static ref DESCRIPTORS: Descriptors = Descriptors::default();
}

#[derive(Debug, Default)]
pub struct Descriptors {
    pub tables: HashSet<TableDescriptor>,
}

impl Descriptors {
    pub fn register(&mut self, table: TableDescriptor) {
        assert!(
            self.tables.insert(table),
            "encountered coliding table descriptors {table:?}"
        );
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct TableDescriptor {
    pub schema: &'static str,
    pub table: &'static str,
    pub primary_key: ColumnDescriptor,
    pub foreign_keys: &'static [ColumnDescriptor],
    pub data: &'static [ColumnDescriptor],
}

impl Hash for TableDescriptor {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.schema.hash(state);
        self.table.hash(state);
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ColumnDescriptor {
    pub name: &'static str,
    pub data_type: DataType,
    pub col_type: ColType,
}
