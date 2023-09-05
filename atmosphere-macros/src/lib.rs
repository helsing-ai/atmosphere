#![allow(unused)]

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

mod schema;
mod sql;

use schema::table::Table;

#[proc_macro_derive(Schema, attributes(primary_key, foreign_key))]
pub fn schema(input: TokenStream) -> TokenStream {
    quote!().into()
}

// ----------------------------------------------------------------------------

// Markers

#[proc_macro_attribute]
pub fn table(attr: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let Data::Struct(DataStruct {
        fields: Fields::Named(FieldsNamed { named: columns, .. }),
        ..
    }) = &input.data
    else {
        panic!("Only named structs can be tables");
    };

    let table = Table::parse(&input, &columns);

    let tid = ("public", table.ident.to_string());

    let table_impl = table.quote_table_impl();
    let read_impl = table.quote_read_impl();

    quote! {
        #[derive(::atmosphere::prelude::sqlx::FromRow)]
        #input
        #table_impl
        #read_impl
    }
    .into()
}

#[proc_macro_attribute]
pub fn relation(attr: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    //let params = parse_macro_input!(attr as syn::AttributeArgs);

    let struct_name = &input.ident;

    let expanded = quote! {
        #input
    };

    expanded.into()
}

// ----------------------------------------------------------------------------

// Query

//#[proc_macro_attribute]
//pub fn query(attr: TokenStream, item: TokenStream) -> TokenStream {
//let query = parse_macro_input!(item as syn::Item);

//let params = parse_macro_input!(attr as syn::LitStr);

//dbg!(params.value().trim());

//dbg!(query.clone().into_token_stream().to_string());

//let expanded = quote! { fn query(&self) {} };

//// 1. analyze signature and infer sqlx function
////      - fetch_one, execute and so on
//// 2. pass sql string to handlebars and Bind:
////      - database tables (smh)
////      - function arguments
////      - replace "{*}" with concrete columns
//// 3. modify signature to be generic over executor and add executor arg
//// 4. populate function body / execute sql in body

//expanded.into()
//}
