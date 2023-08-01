use std::collections::HashMap;
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

#[derive(Clone)]
pub struct Table {
    pub ident: Ident,
    pub schema: String,
    pub table: String,
    pub id: Column,
    pub refs: Vec<Reference>,
    pub data: Vec<Column>,
}

impl Table {
    pub fn parse(input: &DeriveInput, fields: &Punctuated<Field, Comma>) -> Self {
        let ident = &input.ident;

        let columns = fields.iter().map(Column::parse);

        let (id, data): (Vec<Column>, Vec<Column>) = columns.partition(|c| c.id);

        let id = {
            if id.len() == 0 {
                panic!("missing primary id column (#[id]) on table {}", ident);
            }
            if id.len() > 1 {
                panic!(
                    "found more than one primary id column (#[id]) on table {}",
                    ident
                );
            }
            id[0].clone()
        };

        let data = data.into_iter().filter(|d| !d.fk).collect();
        let refs: Vec<Reference> = fields.iter().filter_map(Reference::parse).collect();

        Self {
            ident: ident.to_owned(),
            schema: "public".to_owned(),
            table: ident.to_string().to_lowercase(),
            id,
            refs,
            data,
        }
    }

    pub fn quote_table_impl(&self) -> TokenStream2 {
        let Self {
            ident,
            schema,
            table,
            id,
            refs,
            data,
        } = self;

        let id_ty = &self.id.ty;
        let id = self.id.quote();
        let refs = self.refs.iter().map(|r| r.column.quote());
        let data = self.data.iter().map(|d| d.quote());

        quote!(
            #[automatically_derived]
            impl ::atmosphere::Table for #ident {
                type Id = #id_ty;

                const ID: ::atmosphere::Column<#ident> = #id;

                const SCHEMA: &'static str = #schema;
                const TABLE: &'static str = #table;

                const REFS: &'static [::atmosphere::Column<#ident>] = &[
                    #(#refs),*
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
            id,
            refs,
            data,
        } = self;

        let all = self.select().into_sql();

        let find = {
            let mut query = self.select();
            query.push(format!("WHERE\n  {} = $1", id.name));
            query.into_sql()
        };

        quote!(
            #[automatically_derived]
            #[::atmosphere::prelude::async_trait]
            impl ::atmosphere::Read for #ident {
                async fn find(id: &Self::Id, pool: &::sqlx::PgPool) -> ::atmosphere_core::Result<Self> {
                    ::sqlx::query_as!(
                        Self,
                        #find,
                        id
                    )
                    .fetch_one(pool)
                    .await
                    .map_err(|_| ())
                }

                async fn all(pool: &::sqlx::PgPool) -> ::atmosphere_core::Result<Vec<Self>> {
                    ::sqlx::query_as!(
                        Self,
                        #all
                    )
                    .fetch_all(pool)
                    .await
                    .map_err(|_| ())
                }
            }
        )
    }

    pub fn quote_write_impl(&self) -> TokenStream2 {
        let Self {
            ident,
            schema,
            table,
            id,
            refs,
            data,
        } = self;

        let field_bindings = {
            let mut fields = vec![];

            let name = &id.name;
            fields.push(quote!(&self.#name as _));

            for r in refs {
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
            query.push(format!("\nWHERE\n  {} = $1", id.name));
            query.into_sql()
        };

        let delete = {
            let mut query = self.delete();
            query.push(format!("\nWHERE\n  {} = $1", id.name));
            query.into_sql()
        };

        let delete_field = {
            let name = &id.name;

            quote!(&self.#name)
        };

        dbg!(&save, &update, &delete);

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
            id,
            refs,
            data,
        } = self;

        let mut query = sqlx::QueryBuilder::<sqlx::Postgres>::new("SELECT\n");

        let mut separated = query.separated(",\n  ");

        separated.push(format!("  {} as \"{}: _\"", id.name, id.name));

        for r in refs {
            separated.push(format!("{} as \"{}: _\"", r.column.name, r.column.name));
        }

        for data in data {
            separated.push(format!("{} as \"{}: _\"", data.name, data.name));
        }

        query.push(format!("\nFROM\n  {}\n", self.escaped_table()));

        query
    }

    /// Generate the update statement
    pub fn update(&self) -> QueryBuilder<Postgres> {
        let Self {
            ident,
            schema,
            table,
            id,
            refs,
            data,
        } = self;

        let mut query =
            QueryBuilder::<Postgres>::new(format!("UPDATE {} SET\n  ", self.escaped_table()));

        let mut separated = query.separated(",\n  ");

        let mut col = 2;

        for r in refs {
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
            id,
            refs,
            data,
        } = self;

        let mut query =
            QueryBuilder::<Postgres>::new(format!("INSERT INTO {} (\n  ", self.escaped_table()));

        let mut separated = query.separated(",\n  ");

        separated.push(id.name.to_string());

        for r in refs {
            separated.push(r.column.name.to_string());
        }

        for data in data {
            separated.push(data.name.to_string());
        }

        separated.push_unseparated("\n) VALUES (\n");

        separated.push_unseparated("  $1");

        let cols = 1 + refs.len() + data.len();

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
    pub id: bool,
    pub fk: bool,
    pub name: Ident,
    pub ty: syn::Type,
}

impl Column {
    pub fn parse(field: &Field) -> Self {
        let id = field.attrs.iter().any(|a| a.path.is_ident("id"));
        let fk = field.attrs.iter().any(|a| a.path.is_ident("reference"));

        if id && fk {
            panic!(
                "{} can not be primary key and foreign key at the same time",
                field.ident.as_ref().unwrap()
            );
        }

        Self {
            id,
            fk,
            name: field.ident.clone().unwrap(),
            ty: field.ty.clone(),
        }
    }

    pub fn quote(&self) -> TokenStream2 {
        let Column { id, fk, name, ty } = self;

        let name = name.to_string();

        let col_type = if *id {
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

#[derive(Clone)]
pub struct Reference {
    pub table: Ident,
    pub column: Column,
}

impl Reference {
    pub fn parse(field: &Field) -> Option<Self> {
        let referenced = field
            .attrs
            .iter()
            .filter(|a| a.path.is_ident("reference"))
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
