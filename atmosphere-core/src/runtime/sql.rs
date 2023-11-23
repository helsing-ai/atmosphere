use std::fmt;

use sqlx::QueryBuilder;

use crate::{
    query::{self, Query},
    Bind, Column,
};

pub struct Bindings<T: Bind>(Vec<Column<T>>);

impl<T: Bind> PartialEq for Bindings<T> {
    fn eq(&self, other: &Self) -> bool {
        if self.0.len() != other.0.len() {
            return false;
        }

        for (i, a) in self.0.iter().enumerate() {
            let Some(b) = other.0.get(i) else {
                return false;
            };

            if a.name() != b.name() {
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
            f.field(&c.name());
        }

        f.finish()
    }
}

impl<T: Bind> Bindings<T> {
    pub fn columns(&self) -> &[Column<T>] {
        &self.0
    }

    pub fn empty() -> Self {
        Self(vec![])
    }
}

fn table<T: Bind>() -> String {
    format!("\"{}\".\"{}\"", T::SCHEMA, T::TABLE)
}

/// `SELECT * FROM .. WHERE .. = $1`
pub fn select<T: Bind>() -> Query<T> {
    select_by(Column::PrimaryKey(&T::PRIMARY_KEY))
}

/// `SELECT * FROM .. WHERE .. = $1`
pub fn select_by<T: Bind>(c: Column<T>) -> Query<T> {
    let mut query = QueryBuilder::new("SELECT\n  ");

    let mut separated = query.separated(",\n  ");

    separated.push(T::PRIMARY_KEY.name);

    for ref fk in T::FOREIGN_KEYS {
        separated.push(fk.name);
    }

    for ref data in T::DATA_COLUMNS {
        separated.push(data.name);
    }

    for ref meta in T::META_COLUMNS {
        separated.push(meta.name);
    }

    query.push(format!("\nFROM\n  {}\n", table::<T>()));
    query.push(format!("WHERE {} = $1", c.name()));

    Query::new(
        query::Operation::Select,
        query::Cardinality::One,
        query,
        Bindings(vec![c]),
    )
}

/// `SELECT * FROM ..`
pub fn select_all<T: Bind>() -> Query<T> {
    let mut query = QueryBuilder::new("SELECT\n  ");

    let mut separated = query.separated(",\n  ");

    separated.push(T::PRIMARY_KEY.name);

    for ref fk in T::FOREIGN_KEYS {
        separated.push(fk.name);
    }

    for ref data in T::DATA_COLUMNS {
        separated.push(data.name);
    }

    for ref meta in T::META_COLUMNS {
        separated.push(meta.name);
    }

    query.push(format!("\nFROM\n  {}\n", table::<T>()));

    Query::new(
        query::Operation::Select,
        query::Cardinality::Many,
        query,
        Bindings::empty(),
    )
}

/// `INSERT INTO .. VALUES ..`
pub fn insert<T: Bind>() -> Query<T> {
    let mut builder = QueryBuilder::new(format!("INSERT INTO {}\n  (", table::<T>()));

    let mut bindings = vec![];

    let mut separated = builder.separated(", ");

    separated.push(T::PRIMARY_KEY.name.to_string());
    bindings.push(Column::PrimaryKey(&T::PRIMARY_KEY));

    for fk in T::FOREIGN_KEYS {
        separated.push(fk.name.to_string());
        bindings.push(Column::ForeignKey(fk));
    }

    for data in T::DATA_COLUMNS {
        separated.push(data.name.to_string());
        bindings.push(Column::DataColumn(data));
    }

    for meta in T::META_COLUMNS {
        separated.push(meta.name.to_string());
        bindings.push(Column::MetaColumn(meta));
    }

    separated.push_unseparated(")\nVALUES\n  (");

    separated.push_unseparated("$1");

    let columns = 1 + T::FOREIGN_KEYS.len() + T::DATA_COLUMNS.len() + T::META_COLUMNS.len();

    for c in 2..=columns {
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

/// `UPDATE .. SET .. WHERE ..`
pub fn update<T: Bind>() -> Query<T> {
    let mut builder = QueryBuilder::new(format!("UPDATE {} SET\n  ", table::<T>()));
    let mut bindings = vec![];

    let mut separated = builder.separated(",\n  ");

    separated.push(format!("{} = $1", T::PRIMARY_KEY.name));
    bindings.push(Column::PrimaryKey(&T::PRIMARY_KEY));

    let mut col = 2;

    for ref fk in T::FOREIGN_KEYS {
        separated.push(format!("{} = ${col}", fk.name));
        bindings.push(Column::ForeignKey(fk));
        col += 1;
    }

    for ref data in T::DATA_COLUMNS {
        separated.push(format!("{} = ${col}", data.name));
        bindings.push(Column::DataColumn(data));
        col += 1;
    }

    for ref meta in T::META_COLUMNS {
        separated.push(format!("{} = ${col}", meta.name));
        bindings.push(Column::MetaColumn(meta));
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

/// `UPDATE .. SET .. WHERE .. ON CONFLICT .. DO UPDATE SET`
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

    for ref data in T::DATA_COLUMNS {
        separated.push(format!("{} = EXCLUDED.{}", data.name, data.name));
    }

    for ref meta in T::META_COLUMNS {
        separated.push(format!("{} = EXCLUDED.{}", meta.name, meta.name));
    }

    Query::new(
        query::Operation::Upsert,
        query::Cardinality::One,
        builder,
        bindings,
    )
}

/// `DELETE FROM .. WHERE ..`
pub fn delete<T: Bind>() -> Query<T> {
    delete_by(T::PRIMARY_KEY.as_col())
}

/// `DELETE FROM .. WHERE ..`
pub fn delete_by<T: Bind>(c: Column<T>) -> Query<T> {
    let mut builder = QueryBuilder::new(format!("DELETE FROM {} WHERE ", table::<T>()));

    builder.push(c.name());
    builder.push(" = $1");

    Query::new(
        query::Operation::Delete,
        query::Cardinality::One,
        builder,
        Bindings(vec![Column::PrimaryKey(&T::PRIMARY_KEY)]),
    )
}

#[cfg(test)]
mod tests {
    use crate::{
        runtime::sql::{self, Bindings},
        Bind, Bindable, Column, DataColumn, ForeignKey, MetaColumn, PrimaryKey, Table,
    };

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

        const PRIMARY_KEY: PrimaryKey<Self> = PrimaryKey::new("id");
        const FOREIGN_KEYS: &'static [ForeignKey<Self>] = &[ForeignKey::new("fk")];
        const DATA_COLUMNS: &'static [DataColumn<Self>] = &[DataColumn::new("data")];
        const META_COLUMNS: &'static [MetaColumn<Self>] = &[];

        fn pk(&self) -> &Self::PrimaryKey {
            &self.id
        }
    }

    impl Bind for TestTable {
        fn bind<'q, Q: Bindable<'q>>(&'q self, c: &'q Column<Self>, query: Q) -> crate::Result<Q> {
            match c.name() {
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
            "SELECT\n  id,\n  fk,\n  data\nFROM\n  \"public\".\"test\"\nWHERE id = $1"
        );

        assert_eq!(
            bindings,
            Bindings(vec![Column::PrimaryKey(&TestTable::PRIMARY_KEY),])
        );
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
                Column::PrimaryKey(&TestTable::PRIMARY_KEY),
                Column::ForeignKey(&TestTable::FOREIGN_KEYS[0]),
                Column::DataColumn(&TestTable::DATA_COLUMNS[0]),
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
                Column::PrimaryKey(&TestTable::PRIMARY_KEY),
                Column::ForeignKey(&TestTable::FOREIGN_KEYS[0]),
                Column::DataColumn(&TestTable::DATA_COLUMNS[0]),
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
                Column::PrimaryKey(&TestTable::PRIMARY_KEY),
                Column::ForeignKey(&TestTable::FOREIGN_KEYS[0]),
                Column::DataColumn(&TestTable::DATA_COLUMNS[0]),
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
        assert_eq!(
            bindings,
            Bindings(vec![Column::PrimaryKey(&TestTable::PRIMARY_KEY),])
        );
    }
}
