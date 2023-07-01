use proc_macro::{self, TokenStream};
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{
    parse_macro_input, Attribute, Data, DataStruct, DeriveInput, Expr, ExprLit, Field, Fields,
    FieldsNamed, Ident, Lit, LitStr, Meta, MetaNameValue,
};

#[proc_macro_derive(Model, attributes(schema, table))]
pub fn derive(input: TokenStream) -> TokenStream {
    let DeriveInput {
        ident, data, attrs, ..
    } = parse_macro_input!(input);

    let Data::Struct(model) = data else {
        panic!("unsupported usage of the model dervice macro");
    };

    dbg!(ident);

    //let config = Config::new(&attrs, &ident, &named);
    //let static_model_schema = build_static_model_schema(&config);
    //let sqlx_crud_impl = build_sqlx_crud_impl(&config);

    quote! {
    //#static_model_schema
    //#sqlx_crud_impl
    }
    .into()
}
