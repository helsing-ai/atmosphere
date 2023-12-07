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

use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, ItemStruct};

mod derive;
mod hooks;
mod schema;

use schema::table::Table;

/// A derive macro that processes structs to automatically generate schema-related code. It reads
/// custom attributes and derives necessary traits and implementations for interacting with the
/// database.
///
/// Entity attributes:
///
/// - `#[table(schema = "schema_name", name = "table_name")]` - Set schema and table name
///
/// Field attributes:
///
/// - `#[sql(pk)]` - Mark a column as primary key
/// - `#[sql(fk -> OtherModel)]` - Mark a column as foreign key on `OtherModel`
/// - `#[sql(unique)]` - Mark a column as unique
/// - `#[sql(timestamp = [create|update|delete])]` - Mark a column as timestamp
/// - `#[sql(.., rename = "renamed_sql_col")]` - Rename a column in the generated sql
///
/// Usage:
///
/// ```
/// # use atmosphere::prelude::*;
/// #[derive(Schema)]
/// #[table(schema = "public", name = "user")]
/// struct User {
///     #[sql(pk)]
///     id: i32,
///     #[sql(unique)]
///     username: String,
/// }
///
/// #[derive(Schema)]
/// #[table(schema = "public", name = "post")]
/// struct Post {
///     #[sql(pk)]
///     id: i32,
///     #[sql(fk -> User, rename = "author_id")]
///     author: i32,
/// }
/// ```
#[proc_macro_derive(Schema, attributes(sql))]
pub fn schema(input: TokenStream) -> TokenStream {
    let table = parse_macro_input!(input as Table);
    derive::all(&table).into()
}

/// An attribute macro that stores metadata about the sql table.
/// Must be used after `#[derive(Schema)]`.
///
/// Keys:
///
/// - `schema` - sets schema name.
/// - `name` - sets table name.
///
/// Usage:
///
/// ```
/// # use atmosphere::prelude::*;
/// # #[derive(Schema)]
/// #[table(schema = "public", name = "user")]
/// # struct User {
/// #     #[sql(pk)]
/// #     id: i32,
/// #     #[sql(unique)]
/// #     username: String,
/// # }
/// ```
#[proc_macro_attribute]
pub fn table(_: TokenStream, input: TokenStream) -> TokenStream {
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
                syn::parse_str(&format!("#[sqlx(rename = \"{}\")]", rename)).unwrap();

            field.attrs.push(rename);
        }
    }

    let model = model.to_token_stream();

    quote! {
        #[derive(::atmosphere::sqlx::FromRow)]
        #model
    }
    .into()
}

/// An attribute macro for registering on a table. Must be used after `#[derive(Schema)]`.
///
/// Takes as argument a type which implements `Hook<Self>` for the entity type.
///
/// Usage:
///
/// ```
/// # use atmosphere::prelude::*;
/// # use atmosphere::hooks::*;
/// #[derive(Schema)]
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
