//! # Macros for Atmosphere
//!
//! This crate provides a set of procedural macros to simplify and automate various tasks in
//! atmosphere. These macros enhance the developer experience by reducing boilerplate,
//! ensuring consistency, and integrating seamlessly with the framework's functionalities.
//!
//! This crate includes macros for deriving schema information from structs, handling table-related
//! attributes, and managing hooks within the framework. The macros are designed to be intuitive
//! and align with the framework's conventions, making them a powerful tool in the application
//! development process.

#![cfg(any(feature = "postgres", feature = "mysql", feature = "sqlite"))]

use proc_macro::TokenStream;
use quote::{ToTokens, quote};
use syn::{ItemStruct, parse_macro_input};

mod derive;
mod hooks;
mod schema;

use schema::table::Table;

/// An attribute macro that stores metadata about the sql table and derives needed traits.
///
/// Keys:
///
/// - `schema` - sets schema name.
/// - `name` - sets table name.
///
/// Usage:
///
/// ```ignore
/// # use atmosphere::prelude::*;
/// #[table(schema = "public", name = "user")]
/// # struct User {
/// #     #[sql(pk)]
/// #     id: i32,
/// #     #[sql(unique)]
/// #     username: String,
/// # }
/// ```
#[proc_macro_attribute]
pub fn table(table_args: TokenStream, input: TokenStream) -> TokenStream {
    let mut model = parse_macro_input!(input as ItemStruct);

    for ref mut field in model.fields.iter_mut() {
        let attribute = field
            .attrs
            .iter()
            .find(|a| a.path().is_ident(schema::column::attribute::PATH));

        let Some(attribute) = attribute else {
            continue;
        };

        let attribute: schema::column::attribute::Attribute = attribute.parse_args().unwrap();

        if let Some(rename) = attribute.renamed {
            struct Extract {
                rename: syn::Attribute,
            }

            impl syn::parse::Parse for Extract {
                fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
                    Ok(Self {
                        rename: input
                            .call(syn::Attribute::parse_outer)?
                            .into_iter()
                            .next()
                            .unwrap(),
                    })
                }
            }

            let Extract { rename } =
                syn::parse_str(&format!("#[sqlx(rename = \"{rename}\")]")).unwrap();

            field.attrs.push(rename);
        }
    }

    let table = match Table::parse_struct(&model, table_args) {
        Ok(table) => table,
        Err(error) => return error.into_compile_error().into(),
    };

    for field in model.fields.iter_mut() {
        field.attrs.retain(|attr| !attr.path().is_ident("sql"));
    }

    let model = model.to_token_stream();
    let derives = derive::all(&table);

    quote! {
        #[derive(::atmosphere::sqlx::FromRow)]
        #model

        #derives
    }
    .into()
}

/// An attribute macro for registering on a table. Must be used with `#[table]` macro.
///
/// Takes as argument a type which implements `Hook<Self>` for the entity type.
///
/// Usage:
///
/// ```ignore
/// # use atmosphere::prelude::*;
/// # use atmosphere::hooks::*;
/// #[table(schema = "public", name = "user")]
/// #[hooks(MyHook)]
/// struct User {
///     #[sql(pk)]
///     id: i32,
///     #[sql(unique)]
///     username: String,
/// }
///
/// struct MyHook;
///
/// impl Hook<User> for MyHook {
///     fn stage(&self) -> HookStage {
///         todo!()
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn hooks(attr: TokenStream, input: TokenStream) -> TokenStream {
    let model = parse_macro_input!(input as ItemStruct);
    let _ = parse_macro_input!(attr as hooks::Hooks);
    quote! { #model }.into()
}
