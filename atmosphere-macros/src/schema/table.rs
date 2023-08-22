use std::collections::HashMap;
use std::fmt;
use std::sync::Mutex;

use proc_macro::{self, Span, TokenStream};
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote, ToTokens};
use sqlx::{Postgres, QueryBuilder};
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{
    parse_macro_input, parse_quote, Attribute, Data, DataStruct, DeriveInput, Expr, ExprLit, Field,
    Fields, FieldsNamed, Ident, Lifetime, Lit, LitStr, Meta, MetaNameValue, Stmt,
};

use crate::sql::query::SelectQuery;

#[derive(Clone, Debug)]
pub struct Table {
    pub ident: Ident,
    pub schema: String,
    pub table: String,
    pub primary_key: Column,
    pub foreign_keys: Vec<ForeignKey>,
    pub data: Vec<Column>,
}

impl Table {
    pub fn parse(input: &DeriveInput, fields: &Punctuated<Field, Comma>) -> Self {
        let ident = &input.ident;

        let columns = fields.iter().map(Column::parse);

        let (pk, data): (Vec<Column>, Vec<Column>) = columns.partition(|c| c.pk);

        let pk = {
            if pk.len() == 0 {
                panic!(
                    "missing primary key column (#[primary_key]) on table {}",
                    ident
                );
            }

            if pk.len() > 1 {
                panic!(
                    "found more than one primary key column (#[primary_key]) on table {}",
                    ident
                );
            }

            pk.first().take().cloned().expect("internal error")
        };

        let data = data.into_iter().filter(|d| !d.fk).collect();
        let foreign_keys: Vec<ForeignKey> = fields.iter().filter_map(ForeignKey::parse).collect();

        Self {
            ident: ident.to_owned(),
            schema: "public".to_owned(),
            table: ident.to_string().to_lowercase(),
            primary_key: pk,
            foreign_keys,
            data,
        }
    }

    pub fn quote_table_impl(&self) -> TokenStream2 {
        let Self {
            ident,
            schema,
            table,
            primary_key,
            foreign_keys,
            data,
        } = self;

        let schema = schema.to_string();
        let pk_ty = &self.primary_key.ty;
        let primary_key = self.primary_key.quote();
        let foreign_keys = self.foreign_keys.iter().map(|r| r.column.quote());
        let data = self.data.iter().map(|d| d.quote());

        quote!(
            #[automatically_derived]
            impl ::atmosphere::Table for #ident {
                type PrimaryKey = #pk_ty;

                const PRIMARY_KEY: ::atmosphere::Column<#ident> = #primary_key;

                const SCHEMA: &'static str = #schema;
                const TABLE: &'static str = #table;

                const FOREIGN_KEYS: &'static [::atmosphere::Column<#ident>] = &[
                    #(#foreign_keys),*
                ];
                const DATA: &'static [::atmosphere::Column<#ident>] = &[
                    #(#data),*
                ];
            }
        )
    }

    pub fn quote_read_impl(&self) -> TokenStream2 {
        let Self {
            ident,
            schema,
            table,
            primary_key,
            foreign_keys,
            data,
        } = self;

        let one = {
            let mut query = self.select();
            query.push(format!("WHERE\n{} = $1", primary_key.name));
            query.into_sql()
        };

        let many = {
            let mut query = self.select();
            query.push(format!("WHERE\n{} = ANY($1)", primary_key.name));
            query.into_sql()
        };

        quote!(
            #[automatically_derived]
            #[::atmosphere::prelude::async_trait]
            impl ::atmosphere::Read for #ident {
                async fn find(pk: &Self::PrimaryKey, pool: &::sqlx::PgPool) -> ::atmosphere_core::Result<Self> {
                    ::sqlx::query_as!(
                        Self,
                        #one,
                        pk
                    )
                    .fetch_one(pool)
                    .await
                    .map_err(|_| ())
                }

                async fn find_many(pks: &[impl AsRef<Self::PrimaryKey>], pool: &::sqlx::PgPool) -> ::atmosphere_core::Result<Vec<Self>> {
                    //::sqlx::query_as!(
                        //Self,
                        //#many,
                        //&keys
                    //)
                    //.fetch_all(pool)
                    //.await
                    //.map_err(|_| ())
                    Err(())
                }
            }
        )
    }

    pub fn quote_write_impl(&self) -> TokenStream2 {
        let Self {
            ident,
            schema,
            table,
            primary_key,
            foreign_keys,
            data,
        } = self;

        let field_bindings = {
            let mut fields = vec![];

            let name = &primary_key.name;
            fields.push(quote!(&self.#name as _));

            for r in foreign_keys {
                let name = &r.column.name;
                fields.push(quote!(&self.#name as _));
            }

            for d in data {
                let name = &d.name;
                fields.push(quote!(&self.#name as _));
            }

            fields
        };

        let save = self.insert().into_sql();

        let update = {
            let mut query = self.update();
            query.push(format!("\nWHERE\n  {} = $1", primary_key.name));
            query.into_sql()
        };

        let delete = {
            let mut query = self.delete();
            query.push(format!("\nWHERE\n  {} = $1", primary_key.name));
            query.into_sql()
        };

        let delete_field = {
            let name = &primary_key.name;

            quote!(&self.#name)
        };

        quote!(
            #[automatically_derived]
            #[::atmosphere::prelude::async_trait]
            impl ::atmosphere::Write for #ident {
                async fn save(&self, pool: &::sqlx::PgPool) -> ::atmosphere_core::Result<()> {
                    ::sqlx::query!(
                        #save,
                        #(#field_bindings),*
                    )
                    .execute(pool)
                    .await
                    .map(|_| ())
                    .map_err(|_| ())
                }

                async fn update(&self, pool: &::sqlx::PgPool) -> ::atmosphere_core::Result<()> {
                    ::sqlx::query!(
                        #update,
                        #(#field_bindings),*
                    )
                    .execute(pool)
                    .await
                    .map(|_| ())
                    .map_err(|_| ())
                }

                async fn delete(&self, pool: &::sqlx::PgPool) -> ::atmosphere_core::Result<()> {
                    ::sqlx::query!(
                        #delete,
                        #delete_field
                    )
                    .execute(pool)
                    .await
                    .map(|_| ())
                    .map_err(|_| ())
                }
            }
        )
    }
}

/// Query Building Related Operations
impl Table {
    pub fn escaped_table(&self) -> String {
        format!("\"{}\".\"{}\"", self.schema, self.table)
    }

    /// Generate the base select statement
    pub fn select(&self) -> sqlx::QueryBuilder<sqlx::Postgres> {
        let Self {
            ident,
            schema,
            table,
            primary_key,
            foreign_keys,
            data,
        } = self;

        let rendered = SelectQuery::new(self.clone()).render();

        dbg!(&rendered);

        let query = sqlx::QueryBuilder::new(rendered);

        //let mut query = sqlx::QueryBuilder::<sqlx::Postgres>::new("SELECT\n");

        //let mut separated = query.separated(",\n  ");

        //separated.push(format!(
        //"  {} as \"{}: _\"",
        //primary_key.name, primary_key.name
        //));

        //for r in foreign_keys {
        //separated.push(format!("{} as \"{}: _\"", r.column.name, r.column.name));
        //}

        //for data in data {
        //separated.push(format!("{} as \"{}: _\"", data.name, data.name));
        //}

        //query.push(format!("\nFROM\n  {}\n", self.escaped_table()));

        query
    }

    /// Generate the update statement
    pub fn update(&self) -> QueryBuilder<Postgres> {
        let Self {
            ident,
            schema,
            table,
            primary_key,
            foreign_keys,
            data,
        } = self;

        let mut query =
            QueryBuilder::<Postgres>::new(format!("UPDATE {} SET\n  ", self.escaped_table()));

        let mut separated = query.separated(",\n  ");

        let mut col = 2;

        for r in foreign_keys {
            separated.push(format!("{} = ${col}", r.column.name));
            col += 1;
        }

        for data in data {
            separated.push(format!("{} = ${col}", data.name));
            col += 1;
        }

        query
    }

    /// Generate the insert statement
    pub fn insert(&self) -> QueryBuilder<Postgres> {
        let Self {
            ident,
            schema,
            table,
            primary_key,
            foreign_keys,
            data,
        } = self;

        let mut query =
            QueryBuilder::<Postgres>::new(format!("INSERT INTO {} (\n  ", self.escaped_table()));

        let mut separated = query.separated(",\n  ");

        separated.push(primary_key.name.to_string());

        for r in foreign_keys {
            separated.push(r.column.name.to_string());
        }

        for data in data {
            separated.push(data.name.to_string());
        }

        separated.push_unseparated("\n) VALUES (\n");

        separated.push_unseparated("  $1");

        let cols = 1 + foreign_keys.len() + data.len();

        for c in 2..=cols {
            separated.push(format!("${c}"));
        }

        separated.push_unseparated(")");

        query
    }

    /// Generate the delete statement without where clause
    pub fn delete(&self) -> QueryBuilder<Postgres> {
        QueryBuilder::<Postgres>::new(format!("DELETE FROM {}", self.escaped_table()))
    }
}

#[derive(Clone)]
pub struct Column {
    pub pk: bool,
    pub fk: bool,
    pub name: Ident,
    pub ty: syn::Type,
}

impl Column {
    pub fn parse(field: &Field) -> Self {
        let pk = field.attrs.iter().any(|a| a.path.is_ident("primary_key"));
        let fk = field.attrs.iter().any(|a| a.path.is_ident("foreign_key"));

        if pk && fk {
            panic!(
                "{} can not be primary key and foreign key at the same time",
                field.ident.as_ref().unwrap()
            );
        }

        Self {
            pk,
            fk,
            name: field.ident.clone().unwrap(),
            ty: field.ty.clone(),
        }
    }

    pub fn quote(&self) -> TokenStream2 {
        let Column { pk, fk, name, ty } = self;

        let name = name.to_string();

        let col_type = if *pk {
            quote!(::atmosphere_core::ColType::PrimaryKey)
        } else if *fk {
            quote!(::atmosphere_core::ColType::ForeignKey)
        } else {
            quote!(::atmosphere_core::ColType::Value)
        };

        quote!(
            ::atmosphere_core::Column::new(
                #name,
                ::atmosphere_core::DataType::Unknown,
                #col_type
            )
        )
    }
}

impl fmt::Debug for Column {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Column")
            .field("pk", &self.pk)
            .field("fk", &self.fk)
            .field("name", &self.name.to_string())
            .field("type", &self.ty.to_token_stream().to_string())
            .finish()
    }
}

#[derive(Clone, Debug)]
pub struct ForeignKey {
    pub table: Ident,
    pub column: Column,
}

impl ForeignKey {
    pub fn parse(field: &Field) -> Option<Self> {
        let referenced = field
            .attrs
            .iter()
            .filter(|a| a.path.is_ident("foreign_key"))
            .map(|a| {
                a.parse_args::<Ident>()
                    .expect("ref requires the table it refers to as parameter")
            })
            .collect::<Vec<Ident>>();

        let table = referenced.get(0)?;

        Some(Self {
            table: table.to_owned(),
            column: Column::parse(&field),
        })
    }
}
