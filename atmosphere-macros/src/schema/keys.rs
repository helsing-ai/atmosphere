use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, Type};

use super::column::{ColumnModifiers, NameSet};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PrimaryKey {
    pub modifiers: ColumnModifiers,
    pub name: NameSet,
    pub ty: Type,
}

impl PrimaryKey {
    pub fn quote(&self) -> TokenStream {
        let field = self.name.field();
        let sql = self.name.sql();

        quote!(::atmosphere::PrimaryKey::new(
            stringify!(#field),
            stringify!(#sql)
        ))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ForeignKey {
    pub on: Ident,
    pub modifiers: ColumnModifiers,
    pub name: NameSet,
    pub ty: Type,
}

impl ForeignKey {
    pub fn quote(&self) -> TokenStream {
        let field = self.name.field();
        let sql = self.name.sql();

        quote!(::atmosphere::ForeignKey::new(
            stringify!(#field),
            stringify!(#sql)
        ))
    }
}
