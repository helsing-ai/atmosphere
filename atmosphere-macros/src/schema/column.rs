use std::{fmt, hash::Hash};

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    parse::{Error, Parse, ParseStream},
    Field, Ident, Type,
};

#[derive(Clone, PartialEq, Eq)]
pub struct Column {
    pub pk: bool,
    pub fk: bool,
    pub name: Ident,
    pub ty: Type,
}

impl Hash for Column {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.to_string().hash(state);
    }
}

impl Parse for Column {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let field = input.call(Field::parse_named)?;

        let name = field.ident.ok_or(syn::Error::new(
            input.span(),
            "only named fields are supported",
        ))?;

        let pk = field.attrs.iter().any(|a| a.path().is_ident("primary_key"));
        let fk = field.attrs.iter().any(|a| a.path().is_ident("foreign_key"));

        if pk && fk {
            return Err(Error::new(
                input.span(),
                format!(
                    "{} can not be primary key and foreign key at the same time",
                    name.to_string()
                ),
            ));
        }

        Ok(Self {
            pk,
            fk,
            name,
            ty: field.ty,
        })
    }
}

impl Column {
    pub fn quote(&self) -> TokenStream {
        let Column { pk, fk, name, .. } = self;

        let name = name.to_string();

        let col_type = if *pk {
            quote!(::atmosphere::ColumnType::PrimaryKey)
        } else if *fk {
            quote!(::atmosphere::ColumnType::ForeignKey)
        } else {
            quote!(::atmosphere::ColumnType::Value)
        };

        quote!(
            ::atmosphere::Column::new(
                #name,
                #col_type
            )
        )
    }
}

impl fmt::Debug for Column {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Column")
            .field("pk", &self.pk)
            .field("fk", &self.fk)
            .field("name", &self.name.to_string())
            .field("type", &self.ty.to_token_stream().to_string())
            .finish()
    }
}
