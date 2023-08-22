use proc_macro2::Ident;
use sqlx::{Postgres, QueryBuilder};

use crate::schema::table::{Column, Table};

type Schema = Vec<Table>;

//pub enum QueryType {
//Insert,
//Select,
//Update,
//Delete,
//Raw,
//}

pub struct SelectQuery {
    template: String,
    table: Table,
    schama: Schema,
}

impl SelectQuery {
    pub fn new(table: Table) -> Self {
        Self {
            template: "SELECT {*} FROM {self}".to_string(),
            table,
            schama: vec![],
        }
    }
}

impl SelectQuery {
    pub fn render(self) -> String {
        let mut template = self.template;

        {
            template = template.replace("{self}", self.table.escaped_table().as_str());

            let mut cols = QueryBuilder::<Postgres>::new("");
            let mut separated = cols.separated(", ");

            separated.push(format!(
                "{} as \"{}: _\"",
                self.table.primary_key.name, self.table.primary_key.name
            ));

            for fk in self.table.foreign_keys {
                separated.push(format!("{} as \"{}: _\"", fk.column.name, fk.column.name));
            }

            for data in self.table.data {
                separated.push(format!("{} as \"{}: _\"", data.name, data.name));
            }

            template = template.replace("{*}", cols.sql());
        }

        template
    }
}
