use std::collections::HashSet;

use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned as _;
use syn::{Error, Fields, Ident, LitStr, Token};

use crate::hooks::Hooks;
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
                    ));
                }
            }

            if !input.peek(Token![,]) {
                break;
            }

            input.parse::<Token![,]>()?;
        }

        let schema = schema.ok_or_else(|| {
            syn::Error::new(input.span(), "`#[table]` requires a value for `schema`")
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

    pub primary_key: PrimaryKey,
    pub foreign_keys: HashSet<ForeignKey>,
    pub data_columns: HashSet<DataColumn>,
    pub timestamp_columns: HashSet<TimestampColumn>,

    pub hooks: Hooks,
}

impl Table {
    pub fn parse_struct(
        item: &syn::ItemStruct,
        table_args: proc_macro::TokenStream,
    ) -> syn::Result<Self> {
        let id: TableId = syn::parse(table_args)?;

        let hooks: Hooks = {
            let attr = item.attrs.iter().find(|attr| attr.path().is_ident("hooks"));

            if let Some(attr) = attr {
                attr.parse_args()?
            } else {
                Hooks::default()
            }
        };

        let ident = &item.ident;

        let fields = match &item.fields {
            Fields::Named(n) => n,
            Fields::Unnamed(_) | Fields::Unit => {
                return Err(Error::new(
                    ident.span(),
                    format!("{ident} must use named fields in order to be used with `table`"),
                ));
            }
        };

        let columns = fields
            .clone()
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
                    item.span(),
                    format!(
                        "{ident} declares more than one column as its primary key – only one is allowed"
                    ),
                ));
            }

            primary_keys.into_iter().next().ok_or(Error::new(
                item.span(),
                format!("{ident} must declare one field as its primary key (using `#[sql(pk)]`"),
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
            ident: ident.clone(),
            id,
            primary_key,
            foreign_keys,
            data_columns,
            timestamp_columns,
            hooks,
        })
    }
}
