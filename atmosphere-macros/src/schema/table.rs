use std::collections::HashSet;

use syn::parse::{Parse, ParseStream};
use syn::{Error, Fields, Generics, Ident, LitStr, Token, Visibility};

use crate::schema::column::{Column, DataColumn, TimestampColumn};
use crate::schema::keys::{ForeignKey, PrimaryKey};

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
    pub vis: Visibility,
    pub generics: Generics,
    pub ident: Ident,

    pub id: TableId,

    pub primary_key: PrimaryKey,

    pub foreign_keys: HashSet<ForeignKey>,
    pub data_columns: HashSet<DataColumn>,
    pub timestamp_columns: HashSet<TimestampColumn>,
}

impl Parse for Table {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let item: syn::ItemStruct = input.parse()?;

        let id: TableId = item
            .attrs
            .iter()
            .find(|attr| attr.path().is_ident("table"))
            .ok_or(syn::Error::new(
                input.span(),
                "You need to use the `#[table]` attribute if you want to derive `Schema`",
            ))?
            .parse_args()?;

        let ident = item.ident;

        let fields = match item.fields {
            Fields::Named(n) => n,
            Fields::Unnamed(_) | Fields::Unit => {
                return Err(Error::new(
                    ident.span(),
                    format!(
                        "{} must use named fields in order to derive `Schema`",
                        ident.to_string()
                    ),
                ))
            }
        };

        let columns = fields
            .named
            .into_iter()
            .map(Column::try_from)
            .collect::<syn::Result<HashSet<Column>>>()?;

        let primary_key = {
            let primary_keys: HashSet<PrimaryKey> = columns
                .iter()
                .filter_map(|c| c.as_primary_key())
                .cloned()
                .collect();

            if primary_keys.len() > 1 {
                return Err(Error::new(
                    input.span(),
                    format!(
                        "{} declares more than one column as its primary key – only one is allowed",
                        ident.to_string()
                    ),
                ));
            }

            primary_keys.into_iter().next().ok_or(Error::new(
                input.span(),
                format!(
                    "{} must declare one field as its primary key (using `#[primary_key]`",
                    ident.to_string()
                ),
            ))?
        };

        let foreign_keys = columns
            .iter()
            .filter_map(|c| c.as_foreign_key())
            .cloned()
            .collect();

        let data_columns = columns
            .iter()
            .filter_map(|c| c.as_data_column())
            .cloned()
            .collect();

        let timestamp_columns = columns
            .iter()
            .filter_map(|c| c.as_timestamp_column())
            .cloned()
            .collect();

        Ok(Self {
            vis: item.vis,
            generics: item.generics,
            ident,
            id,
            primary_key,
            foreign_keys,
            data_columns,
            timestamp_columns,
        })
    }
}
