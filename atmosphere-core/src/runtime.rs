/// Runtime sql code generator
pub mod sql {
    use std::{fmt, marker::PhantomData};

    use sqlx::QueryBuilder;

    use crate::{Bind, Column};

    pub struct Bindings<T: Bind>(Vec<&'static Column<T>>);

    impl<T: Bind> PartialEq for Bindings<T> {
        fn eq(&self, other: &Self) -> bool {
            if self.0.len() != other.0.len() {
                return false;
            }

            for (i, a) in self.0.iter().enumerate() {
                let Some(b) = other.0.get(i) else {
                    return false;
                };

                if a.name != b.name {
                    return false;
                }
            }

            true
        }
    }

    impl<T: Bind> Eq for Bindings<T> {}

    impl<T: Bind> fmt::Debug for Bindings<T> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let mut f = f.debug_tuple("Bindings");

            for c in &self.0 {
                f.field(&c.name);
            }

            f.finish()
        }
    }

    impl<T: Bind> Bindings<T> {
        pub fn columns(&self) -> &[&'static Column<T>] {
            &self.0
        }

        pub fn empty() -> Self {
            Self(vec![])
        }
    }

    pub struct Query<T: Bind>(
        pub(crate) QueryBuilder<'static, T::Database>,
        pub(crate) Bindings<T>,
    );

    impl<T: Bind> Query<T> {
        pub fn sql(&self) -> &str {
            self.0.sql()
        }

        #[cfg(test)]
        pub const fn bindings(&self) -> &Bindings<T> {
            &self.1
        }
    }

    /// Code generator utilizing rust's type / trait system
    pub struct SQL<TABLE: Bind>(PhantomData<TABLE>);

    impl<TABLE: Bind> SQL<TABLE> {
        fn table() -> String {
            format!("\"{}\".\"{}\"", TABLE::SCHEMA, TABLE::TABLE)
        }

        /// Yields a sql `select` statement
        ///
        /// Binds: `pk`
        pub fn select() -> Query<TABLE> {
            let mut query = QueryBuilder::<TABLE::Database>::new("SELECT\n  ");

            let mut separated = query.separated(",\n  ");

            separated.push(TABLE::PRIMARY_KEY.name);

            for ref fk in TABLE::FOREIGN_KEYS {
                separated.push(fk.name);
            }

            for ref data in TABLE::DATA {
                separated.push(data.name);
            }

            query.push(format!("\nFROM\n  {}\n", Self::table()));

            Query(query, Bindings::empty())
        }

        /// Yields a sql `insert` statement
        pub fn insert() -> Query<TABLE> {
            let mut query = QueryBuilder::<'static, TABLE::Database>::new(format!(
                "INSERT INTO {}\n  (",
                Self::table()
            ));

            let mut bindings = vec![];

            let mut separated = query.separated(", ");

            separated.push(TABLE::PRIMARY_KEY.name.to_string());
            bindings.push(&TABLE::PRIMARY_KEY);

            for ref fk in TABLE::FOREIGN_KEYS {
                separated.push(fk.name.to_string());
                bindings.push(fk);
            }

            for ref data in TABLE::DATA {
                separated.push(data.name.to_string());
                bindings.push(data);
            }

            separated.push_unseparated(")\nVALUES\n  (");

            separated.push_unseparated("$1");

            let cols = 1 + TABLE::FOREIGN_KEYS.len() + TABLE::DATA.len();

            for c in 2..=cols {
                separated.push(format!("${c}"));
            }

            query.push(")");

            Query(query, Bindings(bindings))
        }

        /// Yields a sql `update` statement
        ///
        /// Binds: `pk`
        pub fn update() -> Query<TABLE> {
            let mut query =
                QueryBuilder::<TABLE::Database>::new(format!("UPDATE {} SET\n  ", Self::table()));
            let mut bindings = vec![];

            let mut separated = query.separated(",\n  ");

            let mut col = 2;

            separated.push(format!("{} = $1", TABLE::PRIMARY_KEY.name));
            bindings.push(&TABLE::PRIMARY_KEY);

            for ref fk in TABLE::FOREIGN_KEYS {
                separated.push(format!("{} = ${col}", fk.name));
                bindings.push(*fk);
                col += 1;
            }

            for ref data in TABLE::DATA {
                separated.push(format!("{} = ${col}", data.name));
                bindings.push(*data);
                col += 1;
            }

            query.push(format!("\nWHERE\n  {} = $1", TABLE::PRIMARY_KEY.name));

            Query(query, Bindings(bindings))
        }

        /// Yields a sql `update .. on conflict insert` statement
        pub fn upsert() -> Query<TABLE> {
            let Query(mut query, bindings) = Self::insert();

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

            Query(query, bindings)
        }

        /// Yields a sql `delete` statement
        ///
        /// Binds: `pk`
        pub fn delete() -> Query<TABLE> {
            let mut query = QueryBuilder::<TABLE::Database>::new(format!(
                "DELETE FROM {} WHERE ",
                Self::table()
            ));

            query.push(TABLE::PRIMARY_KEY.name);
            query.push(" = $1");

            Query(query, Bindings(vec![&TABLE::PRIMARY_KEY]))
        }
    }

    #[cfg(test)]
    mod tests {
        use crate::{runtime::sql::Bindings, Bind, Bindable, Column, Table};

        #[derive(sqlx::FromRow)]
        #[allow(unused)]
        struct TestTable {
            id: i32,
            fk: i32,
            data: bool,
        }

        impl Table for TestTable {
            type Database = sqlx::Postgres;
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

        impl Bind for TestTable {
            fn bind<'q, Q: Bindable<'q, sqlx::Postgres>>(
                &'q self,
                c: &'q Column<Self>,
                query: Q,
            ) -> crate::Result<Q> {
                match c.name {
                    "id" => {
                        return Ok(query.dyn_bind(&self.id));
                    }
                    "fk" => {
                        return Ok(query.dyn_bind(&self.fk));
                    }
                    "data" => {
                        return Ok(query.dyn_bind(&self.data));
                    }
                    _ => unimplemented!(),
                }
            }
        }

        type SQL = super::SQL<TestTable>;

        #[test]
        fn select() {
            assert_eq!(
                SQL::select().sql(),
                "SELECT\n  id,\n  fk,\n  data\nFROM\n  \"public\".\"test\"\n"
            );

            assert_eq!(*SQL::select().bindings(), Bindings::empty());
        }

        #[test]
        fn insert() {
            assert_eq!(
                SQL::insert().sql(),
                "INSERT INTO \"public\".\"test\"\n  (id, fk, data)\nVALUES\n  ($1, $2, $3)"
            );

            assert_eq!(
                *SQL::insert().bindings(),
                Bindings(vec![
                    &TestTable::PRIMARY_KEY,
                    &TestTable::FOREIGN_KEYS[0],
                    &TestTable::DATA[0]
                ])
            );
        }

        #[test]
        fn update() {
            assert_eq!(
                SQL::update().sql(),
                "UPDATE \"public\".\"test\" SET\n  id = $1,\n  fk = $2,\n  data = $3\nWHERE\n  id = $1"
            );

            assert_eq!(
                *SQL::update().bindings(),
                Bindings(vec![
                    &TestTable::PRIMARY_KEY,
                    &TestTable::FOREIGN_KEYS[0],
                    &TestTable::DATA[0]
                ])
            );
        }

        #[test]
        fn upsert() {
            assert_eq!(
                SQL::upsert().sql(),
                "INSERT INTO \"public\".\"test\"\n  (id, fk, data)\nVALUES\n  ($1, $2, $3)\nON CONFLICT(id)\nDO UPDATE SET\n  fk = EXCLUDED.fk,\n  data = EXCLUDED.data"
            );

            assert_eq!(
                *SQL::upsert().bindings(),
                Bindings(vec![
                    &TestTable::PRIMARY_KEY,
                    &TestTable::FOREIGN_KEYS[0],
                    &TestTable::DATA[0]
                ])
            );
        }

        #[test]
        fn delete() {
            assert_eq!(
                SQL::delete().sql(),
                "DELETE FROM \"public\".\"test\" WHERE id = $1"
            );

            assert_eq!(
                *SQL::delete().bindings(),
                Bindings(vec![&TestTable::PRIMARY_KEY])
            );
        }
    }
}
