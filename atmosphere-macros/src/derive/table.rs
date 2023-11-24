use proc_macro2::TokenStream;
use quote::quote;

use crate::schema::table::Table;

pub fn table(table: &Table) -> TokenStream {
    let Table {
        ident,
        id,
        primary_key,
        foreign_keys,
        data_columns,
        meta_columns,
        ..
    } = table;

    let schema = id.schema.to_string();
    let table_name = id.table.to_string();

    let pk_ty = &table.primary_key.ty;
    let pk_field = &table.primary_key.name.field();

    let primary_key = primary_key.quote();
    let foreign_keys = foreign_keys.iter().map(|r| r.quote());
    let data = data_columns.iter().map(|d| d.quote());
    let meta = meta_columns.iter().map(|d| d.quote());

    quote!(
        #[automatically_derived]
        impl ::atmosphere::Table for #ident {
            type PrimaryKey = #pk_ty;

            const SCHEMA: &'static str = #schema;
            const TABLE: &'static str = #table_name;

            const PRIMARY_KEY: ::atmosphere::PrimaryKey<#ident> = #primary_key;
            const FOREIGN_KEYS: &'static [::atmosphere::ForeignKey<#ident>] = &[#(#foreign_keys),*];
            const DATA_COLUMNS: &'static [::atmosphere::DataColumn<#ident>] = &[#(#data),*];
            const META_COLUMNS: &'static [::atmosphere::MetaColumn<#ident>] = &[#(#meta),*];

            fn pk(&self) -> &Self::PrimaryKey {
                &self.#pk_field
            }
        }
    )
}
