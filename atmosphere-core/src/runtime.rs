/// Runtime sql code generator
pub mod sql {
    use std::marker::PhantomData;

    use sqlx::{Database, QueryBuilder};

    use crate::Table;

    /// Code generator utilizing rust's type / trait system
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

            //query.push

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

    #[cfg(test)]
    mod tests {
        use crate::{Column, Table};

        #[derive(sqlx::FromRow)]
        #[allow(unused)]
        struct TestTable {
            id: i32,
            fk: i32,
            data: bool,
        }

        impl Table for TestTable {
            type PrimaryKey = i32;

            const SCHEMA: &'static str = "public";
            const TABLE: &'static str = "test";
            const PRIMARY_KEY: Column<Self> = Column::new("id", crate::ColumnType::PrimaryKey);
            const FOREIGN_KEYS: &'static [Column<Self>] =
                &[Column::new("fk", crate::ColumnType::ForeignKey)];
            const DATA: &'static [Column<Self>] = &[Column::new("data", crate::ColumnType::Value)];

            fn pk(&self) -> &Self::PrimaryKey {
                &self.id
            }
        }

        type SQL = super::SQL<TestTable, sqlx::Postgres>;

        #[test]
        fn select() {
            assert_eq!(
                SQL::select().sql(),
                "SELECT\n  id,\n  fk,\n  data\nFROM\n  \"public\".\"test\"\n"
            );
        }

        #[test]
        fn insert() {
            assert_eq!(
                SQL::insert().sql(),
                "INSERT INTO \"public\".\"test\"\n  (id, fk, data)\nVALUES\n  ($1, $2, $3)"
            );
        }

        #[test]
        fn update() {
            assert_eq!(
                SQL::update().sql(),
                "UPDATE \"public\".\"test\" SET\n  id = $1,\n  fk = $2,\n  data = $3\nWHERE\n  id = $1"
            );
        }

        #[test]
        fn upsert() {
            assert_eq!(
                SQL::upsert().sql(),
                "INSERT INTO \"public\".\"test\"\n  (id, fk, data)\nVALUES\n  ($1, $2, $3)\nON CONFLICT(id)\nDO UPDATE SET\n  fk = EXCLUDED.fk,\n  data = EXCLUDED.data"
            );
        }

        #[test]
        fn delete() {
            assert_eq!(
                SQL::delete().sql(),
                "DELETE FROM \"public\".\"test\" WHERE id = $1"
            );
        }
    }
}
