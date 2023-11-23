use std::collections::HashSet;

use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{Attribute, Error, Ident, LitStr, Token};

use super::column::Column;

#[derive(Clone, Debug)]
pub struct TableId {
    pub schema: String,
    pub table: String,
}

impl Parse for TableId {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut schema = None;
        let mut table = None;

        while !input.is_empty() {
            let ident: syn::Ident = input.parse()?;
            input.parse::<Token![=]>()?;
            let value: LitStr = input.parse()?;

            match ident.to_string().as_str() {
                "schema" => schema = Some(value.value()),
                "name" => table = Some(value.value()),
                _ => {
                    return Err(syn::Error::new_spanned(
                        ident,
                        "`#[table]` supports only the values `schema` and `name`",
                    ))
                }
            }

            if !input.peek(Token![,]) {
                break;
            }

            input.parse::<Token![,]>()?;
        }

        let schema = schema.ok_or_else(|| {
            syn::Error::new(input.span(), "`#[table]` requies a value for `schema`")
        })?;

        let table = table.ok_or_else(|| {
            syn::Error::new(input.span(), "`#[table]` requires a value for `name`")
        })?;

        Ok(Self { schema, table })
    }
}

#[derive(Clone, Debug)]
pub struct Table {
    pub ident: Ident,
    pub id: TableId,

    pub primary_key: Column,

    pub foreign_keys: HashSet<Column>,
    pub data: HashSet<Column>,
}

impl Parse for Table {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attrs: Vec<Attribute> = input.call(Attribute::parse_outer)?;

        let id: TableId = attrs
            .iter()
            .find(|attr| attr.path().is_ident("table"))
            .ok_or(syn::Error::new(
                input.span(),
                "You need to use the `#[table]` attribute if you want to derive `Schema`",
            ))?
            .parse_args()?;

        let _: Token![struct] = input.parse()?;

        let ident: Ident = input.parse()?;

        let content;
        syn::braced!(content in input);

        let columns: Punctuated<Column, Token![,]> =
            content.parse_terminated(Column::parse, Token![,])?;

        let columns: HashSet<Column> = columns.into_iter().collect();

        let primary_key = columns.iter().find(|c| c.pk).cloned().ok_or(Error::new(
            input.span(),
            format!(
                "{} must declare one field as its primary key (using `#[primary_key]`",
                ident.to_string()
            ),
        ))?;

        // verify that there is only one primary key
        if columns.iter().filter(|c| c.pk).count() > 1 {
            return Err(Error::new(
                input.span(),
                format!(
                    "{} declares more than one column as its primary key – only one is allowed",
                    ident.to_string()
                ),
            ));
        }

        let foreign_keys = columns
            .iter()
            .filter(|c| c.pk == false && c.fk == true)
            .cloned()
            .collect();

        let data = columns
            .iter()
            .filter(|c| c.pk == false && c.fk == false)
            .cloned()
            .collect();

        Ok(Self {
            ident,
            id,
            primary_key,
            foreign_keys,
            data,
        })
    }
}

impl Table {
    pub fn quote_table_impl(&self) -> TokenStream {
        let Self {
            ident,
            id,
            primary_key,
            foreign_keys,
            data,
        } = self;

        let schema = id.schema.to_string();
        let table = id.table.to_string();
        let pk_ty = &self.primary_key.ty;
        let pk_field = &self.primary_key.name;
        let primary_key = self.primary_key.quote();
        let foreign_keys = self.foreign_keys.iter().map(|r| r.quote());
        let data = self.data.iter().map(|d| d.quote());

        quote!(
            #[automatically_derived]
            impl ::atmosphere::Table for #ident {
                type PrimaryKey = #pk_ty;
                type Database = ::sqlx::Postgres;

                const PRIMARY_KEY: ::atmosphere::Column<#ident> = #primary_key;

                const SCHEMA: &'static str = #schema;
                const TABLE: &'static str = #table;

                const FOREIGN_KEYS: &'static [::atmosphere::Column<#ident>] = &[
                    #(#foreign_keys),*
                ];
                const DATA: &'static [::atmosphere::Column<#ident>] = &[
                    #(#data),*
                ];

                fn pk(&self) -> &Self::PrimaryKey {
                    &self.#pk_field
                }
            }
        )
    }

    pub fn quote_bind_impl(&self) -> TokenStream {
        let Self {
            ident,
            id,
            primary_key,
            foreign_keys,
            data,
        } = self;

        let col = Ident::new("col", proc_macro2::Span::call_site());
        let query = Ident::new("query", proc_macro2::Span::call_site());

        let primary_key_bind = {
            let name = &self.primary_key.name;

            quote!(
                if #col.name == Self::PRIMARY_KEY.name {
                    use ::atmosphere::Bindable;

                    return Ok(#query.dyn_bind(&self.#name));
                }
            )
        };

        let foreign_key_binds = {
            let mut stream = TokenStream::new();

            for ref fk in &self.foreign_keys {
                let ident = &fk.name;
                let name = fk.name.to_string();

                stream.extend(quote!(
                if #col.name == #name {
                    use ::atmosphere::Bindable;

                    return Ok(#query.dyn_bind(&self.#ident));
                }
                ));
            }

            stream
        };

        let data_binds = {
            let mut stream = TokenStream::new();

            for ref data in &self.data {
                let ident = &data.name;
                let name = data.name.to_string();

                stream.extend(quote!(
                if #col.name == #name {
                    use ::atmosphere::Bindable;

                    return Ok(#query.dyn_bind(&self.#ident));
                }
                ));
            }

            stream
        };

        quote!(
            #[automatically_derived]
            impl ::atmosphere::Bind for #ident {
                fn bind<
                    'q,
                    Q: ::atmosphere::Bindable<'q, Self::Database>
                >(
                    &'q self,
                    #col: &'q ::atmosphere::Column<Self>,
                    #query: Q
                ) -> ::atmosphere::Result<Q> {
                    #primary_key_bind
                    #foreign_key_binds
                    #data_binds

                    Err(::atmosphere::Error::Bind(
                        ::atmosphere::bind::BindError::Unknown(#col.name)
                    ))
                }
            }
        )
    }
}
