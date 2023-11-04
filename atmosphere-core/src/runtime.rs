pub mod sql {
    use std::marker::PhantomData;

    use sqlx::{Database, QueryBuilder};

    use crate::Table;

    pub struct SQL<TABLE: Table, DB: Database>(PhantomData<TABLE>, PhantomData<DB>);

    impl<TABLE: Table, DB: Database> SQL<TABLE, DB> {
        fn table() -> String {
            format!("\"{}\".\"{}\"", TABLE::SCHEMA, TABLE::TABLE)
        }

        /// Yields a sql `select` statement
        ///
        /// Binds: `pk`
        pub fn select() -> QueryBuilder<'static, DB> {
            let mut query = QueryBuilder::<DB>::new("SELECT\n  ");

            let mut separated = query.separated(",\n  ");

            separated.push(TABLE::PRIMARY_KEY.name);

            for ref fk in TABLE::FOREIGN_KEYS {
                separated.push(fk.name);
            }

            for ref data in TABLE::DATA {
                separated.push(data.name);
            }

            query.push(format!("\nFROM\n  {}\n", Self::table()));

            query
        }

        /// Yields a sql `insert` statement
        ///
        /// Binds: `pk, ..fks, ..data`
        pub fn insert() -> QueryBuilder<'static, DB> {
            let mut query = QueryBuilder::<DB>::new(format!("INSERT INTO {}\n  (", Self::table()));

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

        /// Yields a sql `update` statement
        ///
        /// Binds: `pk`
        pub fn update() -> QueryBuilder<'static, DB> {
            let mut query = QueryBuilder::<DB>::new(format!("UPDATE {} SET\n  ", Self::table()));

            let mut separated = query.separated(",\n  ");

            let mut col = 2;

            separated.push(format!("{} = $1", TABLE::PRIMARY_KEY.name));

            for ref fk in TABLE::FOREIGN_KEYS {
                separated.push(format!("{} = ${col}", fk.name));
                col += 1;
            }

            for ref data in TABLE::DATA {
                separated.push(format!("{} = ${col}", data.name));
                col += 1;
            }

            query.push(format!("\nWHERE\n  {} = $1", TABLE::PRIMARY_KEY.name));

            query
        }

        /// Yields a sql `update .. on conflict insert` statement
        ///
        /// Binds: `pk, ..fks, ..data`
        pub fn upsert() -> QueryBuilder<'static, DB> {
            let mut query = Self::insert();

            query.push("\nON CONFLICT(");
            query.push(TABLE::PRIMARY_KEY.name);
            query.push(")\nDO UPDATE SET\n  ");

            let mut separated = query.separated(",\n  ");

            for ref fk in TABLE::FOREIGN_KEYS {
                separated.push(format!("{} = EXCLUDED.{}", fk.name, fk.name));
            }

            for ref data in TABLE::DATA {
                separated.push(format!("{} = EXCLUDED.{}", data.name, data.name));
            }

            query
        }

        /// Yields a sql `delete` statement
        ///
        /// Binds: `pk`
        pub fn delete() -> QueryBuilder<'static, DB> {
            let mut query =
                QueryBuilder::<DB>::new(format!("DELETE FROM {} WHERE ", Self::table()));

            query.push(TABLE::PRIMARY_KEY.name);
            query.push(" = $1");

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
