pub mod sql {
    use std::marker::PhantomData;

    use sqlx::{Database, QueryBuilder};

    use crate::Table;

    pub struct SQL<TABLE: Table, DB: Database>(PhantomData<TABLE>, PhantomData<DB>);

    impl<TABLE: Table, DB: Database> SQL<TABLE, DB> {
        /// Yields a sql insert statement
        pub fn insert() -> QueryBuilder<'static, DB> {
            let mut query = QueryBuilder::<DB>::new(format!(
                "INSERT INTO \"{}\".\"{}\"\n  (",
                TABLE::SCHEMA,
                TABLE::TABLE
            ));

            let mut separated = query.separated(", ");

            separated.push(TABLE::PRIMARY_KEY.name.to_string());

            for r in TABLE::FOREIGN_KEYS {
                separated.push(r.name.to_string());
            }

            for data in TABLE::DATA {
                separated.push(data.name.to_string());
            }

            separated.push_unseparated(")\nVALUES\n  (");

            separated.push_unseparated("$1");

            let cols = 1 + TABLE::FOREIGN_KEYS.len() + TABLE::DATA.len();

            for c in 2..=cols {
                separated.push(format!("${c}"));
            }

            separated.push_unseparated(")");

            query
        }
    }
}

//use std::{collections::HashSet, hash::Hash};

//use crate::ColType;

//#[derive(Debug, PartialOrd, Ord, PartialEq, Eq)]
//pub enum DataType {}

//lazy_static::lazy_static! {
//pub static ref DESCRIPTORS: Descriptors = Descriptors::default();
//}

//#[derive(Debug, Default)]
//pub struct Descriptors {
//pub tables: HashSet<TableDescriptor>,
//}

//impl Descriptors {
//pub fn register(&mut self, table: TableDescriptor) {
////assert!(
////self.tables.insert(table),
////"encountered coliding table descriptors {table:?}"
////);
//}
//}

//#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
//pub struct TableDescriptor {
//pub schema: &'static str,
//pub table: &'static str,
//pub primary_key: ColumnDescriptor,
//pub foreign_keys: &'static [ColumnDescriptor],
//pub data: &'static [ColumnDescriptor],
//}

//impl Hash for TableDescriptor {
//fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
//self.schema.hash(state);
//self.table.hash(state);
//}
//}

//#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
//pub struct ColumnDescriptor {
//pub name: &'static str,
//pub data_type: DataType,
//pub col_type: ColType,
//}
