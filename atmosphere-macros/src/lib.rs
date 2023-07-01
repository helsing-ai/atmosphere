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

#[proc_macro_derive(Model, attributes(id))]
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

    quote! { #model_trait_impl }.into()
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

        Self {
            ident: ident.to_owned(),
            schema: "public".to_owned(),
            table: ident.to_string().to_lowercase(),
            id,
            refs: vec![],
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

        let id = self.id.quote();
        let data = self.data.iter().map(|d| d.quote());

        quote!(
            #[automatically_derived]
            impl ::atmosphere::Model for #ident {
                type Id = i8;

                const ID: ::atmosphere::Column<#ident> = #id;

                const SCHEMA: &'static str = #schema;
                const TABLE: &'static str = #table;

                const REFS: &'static [::atmosphere::Column<#ident>] = &[];
                const DATA: &'static [::atmosphere::Column<#ident>] = &[
                    #(#data),*
                ];
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
        let id = field
            .attrs
            .iter()
            .any(|a| a.path.get_ident().unwrap().to_string() == "id");

        let fk = field
            .attrs
            .iter()
            .any(|a| a.path.get_ident().unwrap().to_string() == "fk");

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
struct Reference;

//fn build_static_model_schema(config: &Config) -> TokenStream2 {
//let crate_name = &config.crate_name;
//let model_schema_ident = &config.model_schema_ident;
//let table_name = &config.table_name;

//let id_column = config.id_column_ident.to_string();
//let columns_len = config.named.iter().count();
//let columns = config
//.named
//.iter()
//.flat_map(|f| &f.ident)
//.map(|f| LitStr::new(format!("{}", f).as_str(), f.span()));

//let sql_queries = build_sql_queries(config);

//quote! {
//#[automatically_derived]
//static #model_schema_ident: #crate_name::schema::Metadata<'static, #columns_len> = #crate_name::schema::Metadata {
//table_name: #table_name,
//id_column: #id_column,
//columns: [#(#columns),*],
//#sql_queries
//};
//}
//}
