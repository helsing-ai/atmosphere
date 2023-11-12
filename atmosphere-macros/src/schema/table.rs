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
        let pk_field = &self.primary_key.name;
        let primary_key = self.primary_key.quote();
        let foreign_keys = self.foreign_keys.iter().map(|r| r.column.quote());
        let data = self.data.iter().map(|d| d.quote());

        quote!(
            #[automatically_derived]
            impl ::atmosphere::Table for #ident {
                type PrimaryKey = #pk_ty;
                type Database = ::sqlx::Postgres;

                const PRIMARY_KEY: ::atmosphere::Column<#ident> = #primary_key;

                const SCHEMA: &'static str = #schema;
                const TABLE: &'static str = #table;

                const FOREIGN_KEYS: &'static [::atmosphere::Column<#ident>] = &[
                    #(#foreign_keys),*
                ];
                const DATA: &'static [::atmosphere::Column<#ident>] = &[
                    #(#data),*
                ];

                fn pk(&self) -> &Self::PrimaryKey {
                    &self.#pk_field
                }
            }
        )
    }

    pub fn quote_bind_impl(&self) -> TokenStream2 {
        let Self {
            ident,
            schema,
            table,
            primary_key,
            foreign_keys,
            data,
        } = self;

        let databases: [TokenStream2; 4] = [
            quote!(::sqlx::Any),
            quote!(::sqlx::Postgres),
            quote!(::sqlx::MySql),
            quote!(::sqlx::Sqlite),
        ];

        let col = Ident::new("col", proc_macro2::Span::call_site());
        let query = Ident::new("query", proc_macro2::Span::call_site());

        let primary_key_bind = {
            let name = &self.primary_key.name;

            quote!(
                if #col.name == Self::PRIMARY_KEY.name {
                    use ::atmosphere_core::Bindable;

                    return Ok(#query.dyn_bind(&self.#name));
                }
            )
        };

        let foreign_key_binds = {
            let mut stream = TokenStream2::new();

            for ref fk in &self.foreign_keys {
                let ident = &fk.column.name;
                let name = fk.column.name.to_string();

                stream.extend(quote!(
                    if #col.name == #name {
                        use ::atmosphere_core::Bindable;

                        return Ok(#query.dyn_bind(&self.#ident));
                    }
                ));
            }

            stream
        };

        let data_binds = {
            let mut stream = TokenStream2::new();

            for ref data in &self.data {
                let ident = &data.name;
                let name = data.name.to_string();

                stream.extend(quote!(
                    if #col.name == #name {
                        use ::atmosphere_core::Bindable;

                        return Ok(#query.dyn_bind(&self.#ident));
                    }
                ));
            }

            stream
        };

        quote!(
            #[automatically_derived]
            impl ::atmosphere::Bind for #ident {
                fn bind<
                    'q,
                    Q: ::atmosphere::Bindable<'q, Self::Database>
                >(
                    &'q self,
                    #col: &'q ::atmosphere::Column<Self>,
                    #query: Q
                ) -> ::atmosphere::Result<Q> {
                    #primary_key_bind
                    #foreign_key_binds
                    #data_binds

                    Err(())
                }
            }
        )
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
            quote!(::atmosphere_core::ColumnType::PrimaryKey)
        } else if *fk {
            quote!(::atmosphere_core::ColumnType::ForeignKey)
        } else {
            quote!(::atmosphere_core::ColumnType::Value)
        };

        quote!(
            ::atmosphere_core::Column::new(
                #name,
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
