#![allow(unused)]

use proc_macro::{self, TokenStream};
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote, ToTokens};
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{
    parse_macro_input, Attribute, Data, DataStruct, DeriveInput, Expr, ExprLit, Field, Fields,
    FieldsNamed, Ident, Lifetime, Lit, LitStr, Meta, MetaNameValue,
};

#[proc_macro_derive(Model, attributes(id, reference))]
pub fn model(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let Data::Struct(DataStruct {
        fields: Fields::Named(FieldsNamed {
            named: columns,
            ..
        }),
        ..
    }) = &input.data else {
        panic!("only named structs can derive the model trait");
    };

    let model = Model::parse(&input, &columns);

    let model_trait_impl = model.quote_trait_impl();
    let read_impl = model.quote_read_impl();

    quote! {
        #model_trait_impl
        #read_impl
    }
    .into()
}

#[derive(Clone)]
struct Model {
    ident: Ident,
    schema: String,
    table: String,
    id: Column,
    refs: Vec<Reference>,
    data: Vec<Column>,
}

impl Model {
    fn parse(input: &DeriveInput, fields: &Punctuated<Field, Comma>) -> Self {
        let ident = &input.ident;

        let columns = fields.iter().map(Column::parse);

        let (id, data): (Vec<Column>, Vec<Column>) = columns.partition(|c| c.id);

        let id = {
            if id.len() == 0 {
                panic!("missing primary id column (#[id]) on model {}", ident);
            }
            if id.len() > 1 {
                panic!(
                    "found more than one primary id column (#[id]) on model {}",
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

    fn quote_trait_impl(&self) -> TokenStream2 {
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
            impl ::atmosphere::Model for #ident {
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

    fn quote_read_impl(&self) -> TokenStream2 {
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

        drop(separated);

        query.push("\nFROM\n");
        query.push(format!("  \"{schema}\".\"{table}\"\n"));

        let fetch_all = query.sql().to_owned();

        query.push(format!("WHERE\n  {} = $1", id.name));

        let fetch_by_id = query.sql().to_owned();

        println!("{}", query.sql());

        quote!(
            #[automatically_derived]
            #[::atmosphere::prelude::async_trait]
            impl ::atmosphere::Read for #ident {
                async fn find(id: &Self::Id, pool: &::sqlx::PgPool) -> ::atmosphere_core::Result<Self> {
                    ::sqlx::query_as!(
                        Self,
                        #fetch_by_id,
                        id
                    )
                    .fetch_one(pool)
                    .await
                    .map_err(|_| ())
                }

                async fn all(pool: &::sqlx::PgPool) -> ::atmosphere_core::Result<Vec<Self>> {
                    ::sqlx::query_as!(
                        Self,
                        #fetch_all
                    )
                    .fetch_all(pool)
                    .await
                    .map_err(|_| ())
                }
            }
        )
    }
}

#[derive(Clone)]
struct Column {
    id: bool,
    fk: bool,
    name: Ident,
    ty: syn::Type,
}

impl Column {
    fn parse(field: &Field) -> Self {
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

    fn quote(&self) -> TokenStream2 {
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
struct Reference {
    model: Ident,
    column: Column,
}

impl Reference {
    fn parse(field: &Field) -> Option<Self> {
        let referenced = field
            .attrs
            .iter()
            .filter(|a| a.path.is_ident("reference"))
            .map(|a| {
                a.parse_args::<Ident>()
                    .expect("ref requires the model it refers to as parameter")
            })
            .collect::<Vec<Ident>>();

        let model = referenced.get(0)?;

        Some(Self {
            model: model.to_owned(),
            column: Column::parse(&field),
        })
    }
}
