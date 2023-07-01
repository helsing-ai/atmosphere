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

    let model_impl = model_impl(&input, &columns).unwrap();

    //dbg!(&model_impl);

    //dbg!(&attrs.get(0).unwrap().tokens);

    //let config = Config::new(&attrs, &ident, &named);
    //let static_model_schema = build_static_model_schema(&config);
    //let sqlx_crud_impl = build_sqlx_crud_impl(&config);

    quote! { #model_impl }.into()
}

fn model_impl(input: &DeriveInput, fields: &Punctuated<Field, Comma>) -> syn::Result<TokenStream2> {
    let ident = &input.ident;

    //let container_attributes = parse_container_attributes(&input.attrs)?;

    //let reads: Vec<Stmt> = fields
    //.iter()
    //.filter_map(|field| -> Option<Stmt> {
    //let id = &field.ident.as_ref()?;
    //let attributes = parse_child_attributes(&field.attrs).unwrap();
    //let ty = &field.ty;

    //if attributes.skip {
    //return Some(parse_quote!(
    //let #id: #ty = Default::default();
    //));
    //}

    //let expr: Expr = match (attributes.flatten, attributes.try_from) {
    //(true, None) => {
    //predicates.push(parse_quote!(#ty: ::sqlx::FromRow<#lifetime, R>));
    //parse_quote!(<#ty as ::sqlx::FromRow<#lifetime, R>>::from_row(row))
    //}
    //(false, None) => {
    //predicates
    //.push(parse_quote!(#ty: ::sqlx::decode::Decode<#lifetime, R::Database>));
    //predicates.push(parse_quote!(#ty: ::sqlx::types::Type<R::Database>));

    //let id_s = attributes
    //.rename
    //.or_else(|| Some(id.to_string().trim_start_matches("r#").to_owned()))
    //.map(|s| match container_attributes.rename_all {
    //Some(pattern) => rename_all(&s, pattern),
    //None => s,
    //})
    //.unwrap();
    //parse_quote!(row.try_get(#id_s))
    //}
    //(true,Some(try_from)) => {
    //predicates.push(parse_quote!(#try_from: ::sqlx::FromRow<#lifetime, R>));
    //parse_quote!(<#try_from as ::sqlx::FromRow<#lifetime, R>>::from_row(row).and_then(|v| <#ty as ::std::convert::TryFrom::<#try_from>>::try_from(v).map_err(|e| ::sqlx::Error::ColumnNotFound("FromRow: try_from failed".to_string()))))
    //}
    //(false,Some(try_from)) => {
    //predicates
    //.push(parse_quote!(#try_from: ::sqlx::decode::Decode<#lifetime, R::Database>));
    //predicates.push(parse_quote!(#try_from: ::sqlx::types::Type<R::Database>));

    //let id_s = attributes
    //.rename
    //.or_else(|| Some(id.to_string().trim_start_matches("r#").to_owned()))
    //.map(|s| match container_attributes.rename_all {
    //Some(pattern) => rename_all(&s, pattern),
    //None => s,
    //})
    //.unwrap();
    //parse_quote!(row.try_get(#id_s).and_then(|v| <#ty as ::std::convert::TryFrom::<#try_from>>::try_from(v).map_err(|e| ::sqlx::Error::ColumnNotFound("FromRow: try_from failed".to_string()))))
    //}
    //};

    //if attributes.default {
    //Some(parse_quote!(let #id: #ty = #expr.or_else(|e| match e {
    //::sqlx::Error::ColumnNotFound(_) => {
    //::std::result::Result::Ok(Default::default())
    //},
    //e => ::std::result::Result::Err(e)
    //})?;))
    //} else {
    //Some(parse_quote!(
    //let #id: #ty = #expr?;
    //))
    //}
    //})
    //.collect();

    let names = fields.iter().map(|field| &field.ident);

    Ok(quote!(
        #[automatically_derived]
        impl ::atmosphere::Model for #ident {
            type Key = i8;

            const KEY: ::atmosphere::Column<#ident> = Column::new(
                "hi",
                ::atmosphere_core::DataType::Number,
                ::atmosphere_core::ColType::PrimaryKey
            );

            const SCHEMA: &'static str = "public";
            const TABLE: &'static str = "forest";

            const REFS: &'static [::atmosphere::Column<#ident>] = &[];
            const DATA: &'static [::atmosphere::Column<#ident>] = &[];

            //fn from_row(row: &#lifetime R) -> ::sqlx::Result<Self> {
                //#(#reads)*

                //::std::result::Result::Ok(#ident {
                    //#(#names),*
                //})
            //}
        }
    ))
}

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

#[proc_macro_attribute]
pub fn atmosphere(attr: TokenStream, item: TokenStream) -> TokenStream {
    println!("attr: \"{}\"", attr.to_string());
    println!("item: \"{}\"", item.to_string());
    item
}
