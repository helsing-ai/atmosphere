use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

mod schema;

use schema::table::Table;

#[proc_macro_derive(Schema, attributes(primary_key, foreign_key))]
pub fn schema(input: TokenStream) -> TokenStream {
    //let input = parse_macro_input!(input as DeriveInput);

    //let Data::Struct(DataStruct {
    //fields: Fields::Named(FieldsNamed { named: columns, .. }),
    //..
    //}) = &input.data
    //else {
    //panic!("Only named structs can be tables");
    //};

    //let table_attr = input
    //.attrs
    //.iter()
    //.find(|attr| attr.path.is_ident("table"))
    //.expect("You need to use the `#[table]` attribute if you want to derive the Schema");

    //dbg!(table_attr.to_token_stream());

    //let table = <Table as syn::parse::Parse>::parse(input.parse);
    let table = parse_macro_input!(input as Table);

    let table_impl = table.quote_table_impl();
    let bind_impl = table.quote_bind_impl();

    quote! {
        #table_impl
        #bind_impl
    }
    .into()
}

// ----------------------------------------------------------------------------

// Markers

#[proc_macro_attribute]
pub fn table(attr: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    quote! {
        #[derive(::atmosphere::prelude::sqlx::FromRow)]
        #input
    }
    .into()
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
