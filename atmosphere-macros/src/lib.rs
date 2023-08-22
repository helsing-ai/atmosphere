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

mod database;
mod table;

use table::Table;

use crate::database::Schema;

#[proc_macro_derive(Table, attributes(primary_key, foreign_key))]
pub fn table_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let Data::Struct(DataStruct {
        fields: Fields::Named(FieldsNamed { named: columns, .. }),
        ..
    }) = &input.data
    else {
        panic!("Only named structs can derive the table trait");
    };

    let table = Table::parse(&input, &columns);

    let tid = (Schema::Public, table.ident.to_string());

    dbg!(table);

    quote! {}.into()
}

// ----------------------------------------------------------------------------

// Markers

#[proc_macro_attribute]
pub fn table(attr: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    //let params = parse_macro_input!(attr as syn::AttributeArgs);

    let struct_name = &input.ident;

    dbg!(struct_name.to_string());

    let expanded = quote! {
        #input
    };

    expanded.into()
}

#[proc_macro_attribute]
pub fn relation(attr: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    //let params = parse_macro_input!(attr as syn::AttributeArgs);

    let struct_name = &input.ident;

    dbg!("rel", struct_name.to_string());

    let expanded = quote! {
        #input
    };

    expanded.into()
}

// ----------------------------------------------------------------------------

// Query

#[proc_macro_attribute]
pub fn query(attr: TokenStream, item: TokenStream) -> TokenStream {
    let query = parse_macro_input!(item as syn::Item);

    let params = parse_macro_input!(attr as syn::LitStr);

    dbg!(params.value().trim());

    dbg!(query.clone().into_token_stream().to_string());

    let expanded = quote! { fn query(&self) {} };

    // 1. analyze signature and infer sqlx function
    //      - fetch_one, execute and so on
    // 2. pass sql string to handlebars and Bind:
    //      - database tables (smh)
    //      - function arguments
    //      - replace "{*}" with concrete columns
    // 3. modify signature to be generic over executor and add executor arg
    // 4. populate function body / execute sql in body

    expanded.into()
}

//#[proc_macro_derive(Table, attributes(primary_key, foreign_key))]
//pub fn table(input: TokenStream) -> TokenStream {
//let input = parse_macro_input!(input as DeriveInput);

//let Data::Struct(DataStruct {
//fields: Fields::Named(FieldsNamed { named: columns, .. }),
//..
//}) = &input.data
//else {
//panic!("Only named structs can derive the table trait");
//};

//let table = Table::parse(&input, &columns);

//let mut db = (*Database).lock().unwrap();

//let tid = (Schema::Public, table.ident.to_string());

//assert!(
//db.contains_key(&tid) == false,
//"Unable to define the table {}: already exists",
//table.ident.to_string(),
//);

//db.insert(tid, table.clone());

//drop(db);

//let table_impl = table.quote_table_impl();
//let read_impl = table.quote_read_impl();
//let write_impl = table.quote_write_impl();

//quote! {
//#table_impl
//#read_impl
//#write_impl
//}
//.into()
//}

// Query Macros
//#[proc_macro_attribute]
//pub fn query(attr: TokenStream, item: TokenStream) -> TokenStream {
//println!("attr: \"{}\"", attr.to_string());
//println!("item: \"{}\"", item.to_string());

//let mut query = parse_macro_input!(item as syn::ItemFn);

//let pool: syn::FnArg = parse_quote!(pool: &::sqlx::PgPool);
//query.sig.inputs.push(pool);

//let (one, many): (syn::Type, syn::Type) = (parse_quote!(Self), parse_quote!(Vec<Self>));

////let fetch = match query.sig.output {
////syn::ReturnType::Type(_, ref o) if **o == one. => quote!(fetch_one(pool)),
////syn::ReturnType::Type(_, ref m) if **m == many => quote!(fetch_many(pool)),
////_ => panic!("unsupported return type found, only `Self` and `Vec<Self>` are supported"),
////};

//let block = query.block;

//query.block = parse_quote!({
//Ok(#block.fetch_one(pool).await.unwrap())
//});

//quote!(#query).into()
//}

//enum Statement {}

//#[proc_macro]
//pub fn sql(input: TokenStream) -> TokenStream {
//let raw = input.to_string();

//let sql = raw.split(" ");
//let mut sanitized = String::new();
//let mut args: Vec<String> = vec![];

//for word in sql {
//if word.starts_with("$") {
//let arg: String = word.chars().skip(1).collect();

//args.push(arg);

//sanitized.push_str(&format!(" ${}", args.len()));

//continue;
//}

//sanitized.push_str(&format!(" {word}"));
//}

//let query = format!("{sanitized}");

//dbg!(&query);

//quote!(::sqlx::query_as!(
//Self,
//#query,
//#(&#args),*
//))
//.into()
//}
