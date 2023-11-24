use proc_macro2::TokenStream;
use quote::quote;

use crate::schema::table::Table;

mod bindings;
mod queries;
mod relationships;
mod table;

pub fn all(table: &Table) -> TokenStream {
    let bindings = bindings::bindings(table);
    let queries = queries::queries(table);
    let relationships = relationships::relationships(table);
    let table = table::table(table);

    quote!(
        #table

        #bindings

        #queries

        #relationships
    )
}
