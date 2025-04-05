//! # SQL Code Generation
//!
//! This submodule provides essential constructors for SQL queries tailored to the operations
//! performed within the Atmosphere framework. It includes functionalities for dynamically building
//! queries for CRUD (Create, Read, Update, Delete) operations, and managing bindings between SQL
//! queries and table entities.
//!
//! ## Key features:
//!
//! - Query Builders: Functions like `select`, `insert`, `update`, `delete`, and `upsert`, which create SQL
//!   queries for their respective operations. These builders ensure that queries are correctly formatted and
//!   aligned with the structure and constraints of the target table.
//!
//! - Binding Management: The `Bindings` struct and its implementations, which manage the relationship between
//!   table columns and the SQL queries they are bound to. This ensures that queries are executed with the correct
//!   parameters and their values.

use std::fmt;

use sqlx::QueryBuilder;

use crate::{
    Bind, Column,
    query::{self, Query},
};

/// Struct representing bindings for SQL queries.
///
/// `Bindings` is responsible for holding a collection of columns that are bound to a specific SQL query.
/// It encapsulates the necessary details for each column, such as field names and SQL representations,
/// ensuring accurate and efficient binding of data to the query.
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

            if a.field() != b.field() {
                return false;
            }

            if a.sql() != b.sql() {
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
            f.field(&c.field());
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
    #[cfg(not(feature = "sqlite"))]
    return format!("\"{}\".\"{}\"", T::SCHEMA, T::TABLE);

    #[cfg(feature = "sqlite")]
    return format!("\"{}\"", T::TABLE);
}

/// Generates a `SELECT` query to retrieve a single row from the table based on its primary key.
///
/// SQL: `SELECT * FROM .. WHERE .. = $1`
pub fn select<T: Bind>() -> Query<T> {
    select_by(Column::PrimaryKey(&T::PRIMARY_KEY))
}

/// Creates a `SELECT` query to retrieve rows from the table based on a specific column.
///
/// SQL: `SELECT * FROM .. WHERE .. = $1`
pub fn select_by<T: Bind>(c: Column<T>) -> Query<T> {
    let mut query = QueryBuilder::new("SELECT\n  ");

    let mut separated = query.separated(",\n  ");

    separated.push(T::PRIMARY_KEY.sql);

    for fk in T::FOREIGN_KEYS {
        separated.push(fk.sql);
    }

    for data in T::DATA_COLUMNS {
        separated.push(data.sql);
    }

    for meta in T::TIMESTAMP_COLUMNS {
        separated.push(meta.sql);
    }

    query.push(format!("\nFROM\n  {}\n", table::<T>()));
    query.push(format!("WHERE {} = $1", c.sql()));

    Query::new(
        query::Operation::Select,
        query::Cardinality::One,
        query,
        Bindings(vec![c]),
    )
}

/// Constructs a `SELECT` query to fetch all rows from the table.
///
/// SQL: `SELECT * FROM ..`
pub fn select_all<T: Bind>() -> Query<T> {
    let mut query = QueryBuilder::new("SELECT\n  ");

    let mut separated = query.separated(",\n  ");

    separated.push(T::PRIMARY_KEY.sql);

    for fk in T::FOREIGN_KEYS {
        separated.push(fk.sql);
    }

    for data in T::DATA_COLUMNS {
        separated.push(data.sql);
    }

    for meta in T::TIMESTAMP_COLUMNS {
        separated.push(meta.sql);
    }

    query.push(format!("\nFROM\n  {}\n", table::<T>()));

    Query::new(
        query::Operation::Select,
        query::Cardinality::Many,
        query,
        Bindings::empty(),
    )
}

/// Generates an `INSERT` query to add a new row to the table.
///
/// SQL: `INSERT INTO .. VALUES ..`
pub fn insert<T: Bind>() -> Query<T> {
    let mut builder = QueryBuilder::new(format!("INSERT INTO {}\n  (", table::<T>()));

    let mut bindings = vec![];

    let mut separated = builder.separated(", ");

    separated.push(T::PRIMARY_KEY.sql.to_string());
    bindings.push(Column::PrimaryKey(&T::PRIMARY_KEY));

    for fk in T::FOREIGN_KEYS {
        separated.push(fk.sql.to_string());
        bindings.push(Column::ForeignKey(fk));
    }

    for data in T::DATA_COLUMNS {
        separated.push(data.sql.to_string());
        bindings.push(Column::Data(data));
    }

    for meta in T::TIMESTAMP_COLUMNS {
        separated.push(meta.sql.to_string());
        bindings.push(Column::Timestamp(meta));
    }

    separated.push_unseparated(")\nVALUES\n  (");

    separated.push_unseparated("$1");

    let columns = 1 + T::FOREIGN_KEYS.len() + T::DATA_COLUMNS.len() + T::TIMESTAMP_COLUMNS.len();

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

/// Creates an `UPDATE` query to modify an existing row in the table.
///
/// SQL: `UPDATE .. SET .. WHERE ..`
pub fn update<T: Bind>() -> Query<T> {
    let mut builder = QueryBuilder::new(format!("UPDATE {} SET\n  ", table::<T>()));
    let mut bindings = vec![];

    let mut separated = builder.separated(",\n  ");

    separated.push(format!("{} = $1", T::PRIMARY_KEY.sql));
    bindings.push(Column::PrimaryKey(&T::PRIMARY_KEY));

    let mut col = 2;

    for fk in T::FOREIGN_KEYS {
        separated.push(format!("{} = ${col}", fk.sql));
        bindings.push(Column::ForeignKey(fk));
        col += 1;
    }

    for data in T::DATA_COLUMNS {
        separated.push(format!("{} = ${col}", data.sql));
        bindings.push(Column::Data(data));
        col += 1;
    }

    for meta in T::TIMESTAMP_COLUMNS {
        separated.push(format!("{} = ${col}", meta.sql));
        bindings.push(Column::Timestamp(meta));
        col += 1;
    }

    builder.push(format!("\nWHERE\n  {} = $1", T::PRIMARY_KEY.sql));

    Query::new(
        query::Operation::Update,
        query::Cardinality::One,
        builder,
        Bindings(bindings),
    )
}

/// Constructs an `UPSERT` query (update or insert) for a row in the table.
///
/// SQL: `UPDATE .. SET .. WHERE .. ON CONFLICT .. DO UPDATE SET`
pub fn upsert<T: Bind>() -> Query<T> {
    let Query {
        mut builder,
        bindings,
        ..
    } = insert::<T>();

    builder.push("\nON CONFLICT(");
    builder.push(T::PRIMARY_KEY.sql);
    builder.push(")\nDO UPDATE SET\n  ");

    let mut separated = builder.separated(",\n  ");

    for fk in T::FOREIGN_KEYS {
        separated.push(format!("{} = EXCLUDED.{}", fk.sql, fk.sql));
    }

    for data in T::DATA_COLUMNS {
        separated.push(format!("{} = EXCLUDED.{}", data.sql, data.sql));
    }

    for meta in T::TIMESTAMP_COLUMNS {
        separated.push(format!("{} = EXCLUDED.{}", meta.sql, meta.sql));
    }

    Query::new(
        query::Operation::Upsert,
        query::Cardinality::One,
        builder,
        bindings,
    )
}

/// Generates a `DELETE` query to remove a row from the table based on its primary key.
///
/// SQL: `DELETE FROM .. WHERE ..`
pub fn delete<T: Bind>() -> Query<T> {
    delete_by(T::PRIMARY_KEY.as_col())
}

/// Creates a `DELETE` query to remove rows from the table based on a specific column.
///
/// SQL: `DELETE FROM .. WHERE ..`
pub fn delete_by<T: Bind>(c: Column<T>) -> Query<T> {
    let mut builder = QueryBuilder::new(format!("DELETE FROM {} WHERE ", table::<T>()));

    builder.push(c.sql());
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
        Bind, Bindable, Column, DataColumn, ForeignKey, PrimaryKey, Table, TimestampColumn,
        runtime::sql::{self, Bindings},
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

        const PRIMARY_KEY: PrimaryKey<Self> = PrimaryKey::new("id", "id_sql_col");
        const FOREIGN_KEYS: &'static [ForeignKey<Self>] = &[ForeignKey::new("fk", "fk_sql_col")];
        const DATA_COLUMNS: &'static [DataColumn<Self>] =
            &[DataColumn::new("data", "data_sql_col")];
        const TIMESTAMP_COLUMNS: &'static [TimestampColumn<Self>] = &[];

        fn pk(&self) -> &Self::PrimaryKey {
            &self.id
        }
    }

    impl Bind for TestTable {
        fn bind<'q, Q: Bindable<'q>>(&'q self, c: &'q Column<Self>, query: Q) -> crate::Result<Q> {
            match c.field() {
                "id" => Ok(query.dyn_bind(self.id)),
                "fk" => Ok(query.dyn_bind(self.fk)),
                "data" => Ok(query.dyn_bind(self.data)),
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
            "SELECT\n  id_sql_col,\n  fk_sql_col,\n  data_sql_col\nFROM\n  \"public\".\"test\"\nWHERE id_sql_col = $1"
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
            "INSERT INTO \"public\".\"test\"\n  (id_sql_col, fk_sql_col, data_sql_col)\nVALUES\n  ($1, $2, $3)"
        );

        assert_eq!(
            bindings,
            Bindings(vec![
                Column::PrimaryKey(&TestTable::PRIMARY_KEY),
                Column::ForeignKey(&TestTable::FOREIGN_KEYS[0]),
                Column::Data(&TestTable::DATA_COLUMNS[0]),
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
            "UPDATE \"public\".\"test\" SET\n  id_sql_col = $1,\n  fk_sql_col = $2,\n  data_sql_col = $3\nWHERE\n  id_sql_col = $1"
        );

        assert_eq!(
            bindings,
            Bindings(vec![
                Column::PrimaryKey(&TestTable::PRIMARY_KEY),
                Column::ForeignKey(&TestTable::FOREIGN_KEYS[0]),
                Column::Data(&TestTable::DATA_COLUMNS[0]),
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
            "INSERT INTO \"public\".\"test\"\n  (id_sql_col, fk_sql_col, data_sql_col)\nVALUES\n  ($1, $2, $3)\nON CONFLICT(id_sql_col)\nDO UPDATE SET\n  fk_sql_col = EXCLUDED.fk_sql_col,\n  data_sql_col = EXCLUDED.data_sql_col"
        );

        assert_eq!(
            bindings,
            Bindings(vec![
                Column::PrimaryKey(&TestTable::PRIMARY_KEY),
                Column::ForeignKey(&TestTable::FOREIGN_KEYS[0]),
                Column::Data(&TestTable::DATA_COLUMNS[0]),
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
            "DELETE FROM \"public\".\"test\" WHERE id_sql_col = $1"
        );
        assert_eq!(
            bindings,
            Bindings(vec![Column::PrimaryKey(&TestTable::PRIMARY_KEY),])
        );
    }
}
