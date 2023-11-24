use proc_macro2::TokenStream;

use crate::schema::table::Table;

mod unique;

pub fn queries(table: &Table) -> TokenStream {
    unique::queries(table)
}
