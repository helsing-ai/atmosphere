use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

use crate::schema::table::Table;

pub fn bindings(table: &Table) -> TokenStream {
    let col = Ident::new("col", proc_macro2::Span::call_site());
    let query = Ident::new("query", proc_macro2::Span::call_site());

    let mut binds = TokenStream::new();

    {
        let field = &table.primary_key.name.field();

        binds.extend(quote!(
            if #col.field() == Self::PRIMARY_KEY.field {
                use ::atmosphere::Bindable;
                return Ok(#query.dyn_bind(&self.#field));
            }
        ));
    }

    for ref fk in &table.foreign_keys {
        let field = fk.name.field();

        binds.extend(quote!(
            if #col.field() == stringify!(#field) {
                use ::atmosphere::Bindable;
                return Ok(#query.dyn_bind(&self.#field));
            }
        ));
    }

    for ref data in &table.data_columns {
        let field = data.name.field();

        binds.extend(quote!(
            if #col.field() == stringify!(#field) {
                use ::atmosphere::Bindable;
                return Ok(#query.dyn_bind(&self.#field));
            }
        ));
    }

    for ref ts in &table.timestamp_columns {
        let field = ts.name.field();

        binds.extend(quote!(
            if #col.field() == stringify!(#field) {
                use ::atmosphere::Bindable;
                return Ok(#query.dyn_bind(&self.#field));
            }
        ));
    }

    let ident = &table.ident;

    quote!(
        #[automatically_derived]
        impl ::atmosphere::Bind for #ident {
            fn bind<
                'q,
                Q: ::atmosphere::Bindable<'q>
            >(
                &'q self,
                #col: &'q ::atmosphere::Column<Self>,
                #query: Q
            ) -> ::atmosphere::Result<Q> {
                #binds

                Err(::atmosphere::Error::Bind(
                    ::atmosphere::bind::BindError::Unknown(#col.field())
                ))
            }
        }
    )
}
