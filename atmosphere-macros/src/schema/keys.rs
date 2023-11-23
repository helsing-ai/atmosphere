use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, Type};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PrimaryKey {
    pub name: Ident,
    pub ty: Type,
}

impl PrimaryKey {
    pub fn quote(&self) -> TokenStream {
        let name = self.name.to_string();

        quote!(
            ::atmosphere::PrimaryKey::new(#name)
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ForeignKey {
    pub foreign_table: Ident,
    pub name: Ident,
    pub ty: Type,
}

impl ForeignKey {
    pub fn quote_dynamic(&self) -> TokenStream {
        let name = self.name.to_string();

        quote!(
            ::atmosphere::ForeignKey::new(#name)
        )
    }
}
