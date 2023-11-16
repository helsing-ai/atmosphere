use std::fmt;

use sqlx::QueryBuilder;

use crate::{
    query::{self, Query},
    Bind, Column,
};

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

fn table<T: Bind>() -> String {
    format!("\"{}\".\"{}\"", T::SCHEMA, T::TABLE)
}

/// Yields a sql `select` statement
pub fn select<T: Bind>() -> Query<T> {
    let mut query = QueryBuilder::<T::Database>::new("SELECT\n  ");

    let mut separated = query.separated(",\n  ");

    separated.push(T::PRIMARY_KEY.name);

    for ref fk in T::FOREIGN_KEYS {
        separated.push(fk.name);
    }

    for ref data in T::DATA {
        separated.push(data.name);
    }

    query.push(format!("\nFROM\n  {}\n", table::<T>()));
    query.push(format!("WHERE {} = $1", T::PRIMARY_KEY.name));

    Query::new(
        query::Operation::Select,
        query::Cardinality::One,
        query,
        Bindings(vec![&T::PRIMARY_KEY]),
    )
}

/// Yields a sql `insert` statement
pub fn insert<T: Bind>() -> Query<T> {
    let mut builder =
        QueryBuilder::<'static, T::Database>::new(format!("INSERT INTO {}\n  (", table::<T>()));

    let mut bindings = vec![];

    let mut separated = builder.separated(", ");

    separated.push(T::PRIMARY_KEY.name.to_string());
    bindings.push(&T::PRIMARY_KEY);

    for ref fk in T::FOREIGN_KEYS {
        separated.push(fk.name.to_string());
        bindings.push(fk);
    }

    for ref data in T::DATA {
        separated.push(data.name.to_string());
        bindings.push(data);
    }

    separated.push_unseparated(")\nVALUES\n  (");

    separated.push_unseparated("$1");

    let cols = 1 + T::FOREIGN_KEYS.len() + T::DATA.len();

    for c in 2..=cols {
        separated.push(format!("${c}"));
    }

    builder.push(")");

    Query::new(
        query::Operation::Insert,
        query::Cardinality::One,
        builder,
        Bindings(bindings),
    )
}

/// Yields a sql `update` statement
pub fn update<T: Bind>() -> Query<T> {
    let mut builder = QueryBuilder::<T::Database>::new(format!("UPDATE {} SET\n  ", table::<T>()));
    let mut bindings = vec![];

    let mut separated = builder.separated(",\n  ");

    let mut col = 2;

    separated.push(format!("{} = $1", T::PRIMARY_KEY.name));
    bindings.push(&T::PRIMARY_KEY);

    for ref fk in T::FOREIGN_KEYS {
        separated.push(format!("{} = ${col}", fk.name));
        bindings.push(*fk);
        col += 1;
    }

    for ref data in T::DATA {
        separated.push(format!("{} = ${col}", data.name));
        bindings.push(*data);
        col += 1;
    }

    builder.push(format!("\nWHERE\n  {} = $1", T::PRIMARY_KEY.name));

    Query::new(
        query::Operation::Update,
        query::Cardinality::One,
        builder,
        Bindings(bindings),
    )
}

/// Yields a sql `update .. on conflict insert` statement
pub fn upsert<T: Bind>() -> Query<T> {
    let Query {
        mut builder,
        bindings,
        ..
    } = insert::<T>();

    builder.push("\nON CONFLICT(");
    builder.push(T::PRIMARY_KEY.name);
    builder.push(")\nDO UPDATE SET\n  ");

    let mut separated = builder.separated(",\n  ");

    for ref fk in T::FOREIGN_KEYS {
        separated.push(format!("{} = EXCLUDED.{}", fk.name, fk.name));
    }

    for ref data in T::DATA {
        separated.push(format!("{} = EXCLUDED.{}", data.name, data.name));
    }

    Query::new(
        query::Operation::Upsert,
        query::Cardinality::One,
        builder,
        bindings,
    )
}

/// Yields a sql `delete` statement
///
/// Binds: `pk`
pub fn delete<T: Bind>() -> Query<T> {
    let mut builder =
        QueryBuilder::<T::Database>::new(format!("DELETE FROM {} WHERE ", table::<T>()));

    builder.push(T::PRIMARY_KEY.name);
    builder.push(" = $1");

    Query::new(
        query::Operation::Delete,
        query::Cardinality::One,
        builder,
        Bindings(vec![&T::PRIMARY_KEY]),
    )
}

#[cfg(test)]
mod tests {
    use crate::{
        runtime::sql::{self, Bindings},
        Bind, Bindable, Column, Table,
    };

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

    #[test]
    fn select() {
        let sql::Query {
            builder, bindings, ..
        } = sql::select::<TestTable>();

        assert_eq!(
            builder.sql(),
            "SELECT\n  id,\n  fk,\n  data\nFROM\n  \"public\".\"test\"\n"
        );

        assert_eq!(bindings, Bindings(vec![&TestTable::PRIMARY_KEY]));
    }

    #[test]
    fn insert() {
        let sql::Query {
            builder, bindings, ..
        } = sql::insert::<TestTable>();

        assert_eq!(
            builder.sql(),
            "INSERT INTO \"public\".\"test\"\n  (id, fk, data)\nVALUES\n  ($1, $2, $3)"
        );

        assert_eq!(
            bindings,
            Bindings(vec![
                &TestTable::PRIMARY_KEY,
                &TestTable::FOREIGN_KEYS[0],
                &TestTable::DATA[0]
            ])
        );
    }

    #[test]
    fn update() {
        let sql::Query {
            builder, bindings, ..
        } = sql::update::<TestTable>();

        assert_eq!(
            builder.sql(),
            "UPDATE \"public\".\"test\" SET\n  id = $1,\n  fk = $2,\n  data = $3\nWHERE\n  id = $1"
        );

        assert_eq!(
            bindings,
            Bindings(vec![
                &TestTable::PRIMARY_KEY,
                &TestTable::FOREIGN_KEYS[0],
                &TestTable::DATA[0]
            ])
        );
    }

    #[test]
    fn upsert() {
        let sql::Query {
            builder, bindings, ..
        } = sql::upsert::<TestTable>();

        assert_eq!(
                builder.sql(),
                "INSERT INTO \"public\".\"test\"\n  (id, fk, data)\nVALUES\n  ($1, $2, $3)\nON CONFLICT(id)\nDO UPDATE SET\n  fk = EXCLUDED.fk,\n  data = EXCLUDED.data"
            );

        assert_eq!(
            bindings,
            Bindings(vec![
                &TestTable::PRIMARY_KEY,
                &TestTable::FOREIGN_KEYS[0],
                &TestTable::DATA[0]
            ])
        );
    }

    #[test]
    fn delete() {
        let sql::Query {
            builder, bindings, ..
        } = sql::delete::<TestTable>();

        assert_eq!(
            builder.sql(),
            "DELETE FROM \"public\".\"test\" WHERE id = $1"
        );
        assert_eq!(bindings, Bindings(vec![&TestTable::PRIMARY_KEY]));
    }
}
