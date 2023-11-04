use crate::{Column, Result, Table};
use sqlx::{database::HasArguments, Database};

type Query<'q, DB> = sqlx::query::Query<'q, DB, <DB as HasArguments<'q>>::Arguments>;

/// Bind columns to SQL Queries
pub trait Bind<DB: Database>: Table
where
    DB: Database,
{
    /// Bind a single column to the query
    fn bind<'q>(&'q self, c: &'q Column<Self>, query: Query<'q, DB>) -> Result<Query<'q, DB>>;

    /// Bind a all columns to the query
    fn bind_all<'q>(&'q self, mut query: Query<'q, DB>) -> Result<Query<'q, DB>> {
        query = self.bind_primary_key(query)?;
        query = self.bind_foreign_keys(query)?;
        query = self.bind_data(query)?;

        Ok(query)
    }

    /// Bind the primary key column to the query
    fn bind_primary_key<'q>(&'q self, query: Query<'q, DB>) -> Result<Query<'q, DB>> {
        self.bind(&Self::PRIMARY_KEY, query)
    }

    /// Bind the foreign keys columns to the query
    fn bind_foreign_keys<'q>(&'q self, mut query: Query<'q, DB>) -> Result<Query<'q, DB>> {
        for ref fk in Self::FOREIGN_KEYS {
            query = self.bind(fk, query)?;
        }

        Ok(query)
    }

    /// Bind the data columns to the query
    fn bind_data<'q>(&'q self, mut query: Query<'q, DB>) -> Result<Query<'q, DB>> {
        for ref data in Self::DATA {
            query = self.bind(data, query)?;
        }

        Ok(query)
    }
}
