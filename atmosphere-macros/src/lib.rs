use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, ItemStruct};

mod derive;
mod hooks;
mod schema;

use schema::table::Table;

#[proc_macro_derive(Schema, attributes(sql, hook))]
pub fn schema(input: TokenStream) -> TokenStream {
    let table = parse_macro_input!(input as Table);
    derive::all(&table).into()
}

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
        #[derive(::atmosphere::prelude::sqlx::FromRow)]
        #model
    }
    .into()
}

#[proc_macro_attribute]
pub fn hooks(attr: TokenStream, input: TokenStream) -> TokenStream {
    let model = parse_macro_input!(input as ItemStruct);
    let _ = parse_macro_input!(attr as hooks::Hooks);
    quote! { #model }.into()
}
