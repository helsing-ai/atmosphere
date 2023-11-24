use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::Ident;

use crate::schema::{column::Column, table::Table};

pub fn queries(table: &Table) -> TokenStream {
    let mut stream = TokenStream::new();

    let ident = &table.ident;

    let fks: Vec<Column> = table
        .foreign_keys
        .iter()
        .filter(|fk| fk.modifiers.unique)
        .cloned()
        .map(|fk| Column::ForeignKey(fk))
        .collect();

    let data: Vec<Column> = table
        .data_columns
        .iter()
        .filter(|data| data.modifiers.unique)
        .cloned()
        .map(|data| Column::DataColumn(data))
        .collect();

    for column in fks.iter().chain(data.iter()) {
        let ty = column.ty();
        let col = column.name().field().to_string().to_lowercase();
        let column = column.quote();

        let find_by_col = Ident::new(&format!("find_by_{col}"), Span::mixed_site());
        let delete_by_col = Ident::new(&format!("delete_by_{col}"), Span::mixed_site());

        stream.extend(quote!(
            #[automatically_derived]
            impl #ident {
                pub async fn #find_by_col<'e, E>(
                    value: &#ty,
                    executor: E
                ) -> ::atmosphere::Result<Option<#ident>>
                where
                    E: ::sqlx::Executor<'e, Database = ::atmosphere::Driver>,
                    for<'q> <::atmosphere::Driver as ::sqlx::database::HasArguments<'q>>::Arguments:
                        ::sqlx::IntoArguments<'q, ::atmosphere::Driver> + Send
                {
                    use ::atmosphere::{
                        query::{Query, QueryError},
                        runtime::sql,
                        Error
                    };

                    const COLUMN: ::atmosphere::Column<#ident> = #column.as_col();

                    let query = sql::select_by::<#ident>(COLUMN.clone());

                    ::sqlx::query_as(query.sql())
                        .bind(value)
                        .persistent(false)
                        .fetch_optional(executor)
                        .await
                        .map_err(QueryError::from)
                        .map_err(Error::Query)
                }

                pub async fn #delete_by_col<'e, E>(
                    value: &#ty,
                    executor: E,
                ) -> ::atmosphere::Result<<::atmosphere::Driver as ::sqlx::Database>::QueryResult>
                where
                    E: ::sqlx::Executor<'e, Database = ::atmosphere::Driver>,
                    for<'q> <::atmosphere::Driver as ::sqlx::database::HasArguments<'q>>::Arguments:
                        ::sqlx::IntoArguments<'q, ::atmosphere::Driver> + Send
                {
                    use ::atmosphere::{
                        query::{Query, QueryError},
                        runtime::sql,
                        Error
                    };

                    const COLUMN: ::atmosphere::Column<#ident> = #column.as_col();

                    let query = sql::delete_by::<#ident>(COLUMN.clone());

                    ::sqlx::query(query.sql())
                        .bind(value)
                        .persistent(false)
                        .execute(executor)
                        .await
                        .map_err(QueryError::from)
                        .map_err(Error::Query)
                }
            }
        ))
    }

    stream
}
