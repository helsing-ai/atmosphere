use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, ItemStruct};

mod schema;

use schema::table::Table;

#[proc_macro_derive(Schema, attributes(sql))]
pub fn schema(input: TokenStream) -> TokenStream {
    let table = parse_macro_input!(input as Table);
    let table_impl = table.quote_table_impl();
    let rel_impl = table.quote_rel_impls();
    let bind_impl = table.quote_bind_impl();

    quote! {
        #table_impl
        #rel_impl
        #bind_impl
    }
    .into()
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
