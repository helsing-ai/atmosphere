use std::hash::Hash;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parse::{Error, Parse, ParseStream},
    Field, Ident, Type,
};

use super::keys::{ForeignKey, PrimaryKey};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ColumnModifiers {
    pub unique: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum MetaColumn {
    CreatedAt { name: Ident, ty: Type },
    UpdatedAt { name: Ident, ty: Type },
    DeletedAt { name: Ident, ty: Type },
}

impl MetaColumn {
    pub fn name(&self) -> &Ident {
        match self {
            Self::CreatedAt { name, .. }
            | Self::UpdatedAt { name, .. }
            | Self::DeletedAt { name, .. } => name,
        }
    }

    pub fn quote(&self) -> TokenStream {
        let name = self.name().to_string();

        unimplemented!()

        //quote!(
        //::atmosphere::MetaColumn::new(#name)
        //)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct DataColumn {
    pub modifiers: ColumnModifiers,
    pub name: Ident,
    pub ty: Type,
}

impl DataColumn {
    pub fn quote(&self) -> TokenStream {
        let name = self.name.to_string();

        quote!(
            ::atmosphere::DataColumn::new(#name)
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Column {
    PrimaryKey(PrimaryKey),
    ForeignKey(ForeignKey),
    DataColumn(DataColumn),
    MetaColumn(MetaColumn),
}

impl Hash for Column {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name().to_string().hash(state);
    }
}

mod attributes {
    pub const PRIMARY_KEY: &str = "primary_key";
    pub const FOREIGN_KEY: &str = "foreign_key";
    pub const UNIQUE: &str = "unique";

    pub const META_CREATED_AT: &str = "created_at";
    pub const META_UPDATED_AT: &str = "updated_at";
    pub const META_DELETED_AT: &str = "deleted_at";
}

impl Parse for Column {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let field = input.call(Field::parse_named)?;

        let name = field.ident.ok_or(syn::Error::new(
            input.span(),
            "only named fields are supported",
        ))?;

        let ty = field.ty;

        let primary_key = field
            .attrs
            .iter()
            .find(|a| a.path().is_ident(attributes::PRIMARY_KEY));

        let foreign_key = field
            .attrs
            .iter()
            .find(|a| a.path().is_ident(attributes::FOREIGN_KEY));

        let unique = field
            .attrs
            .iter()
            .find(|a| a.path().is_ident(attributes::UNIQUE));

        let meta_created = field
            .attrs
            .iter()
            .find(|a| a.path().is_ident(attributes::META_CREATED_AT));

        let meta_updated = field
            .attrs
            .iter()
            .find(|a| a.path().is_ident(attributes::META_DELETED_AT));

        let meta_deleted = field
            .attrs
            .iter()
            .find(|a| a.path().is_ident(attributes::META_DELETED_AT));

        if primary_key.is_some() && foreign_key.is_some() {
            return Err(Error::new(
                input.span(),
                format!(
                    "{} can not be primary key and foreign key at the same time",
                    name.to_string()
                ),
            ));
        }

        if primary_key.is_some() && unique.is_some() {
            return Err(Error::new(
                input.span(),
                format!(
                    "{} uniqueness is inhereted by marking a column as primary key",
                    name.to_string()
                ),
            ));
        }

        if (primary_key.is_some() || foreign_key.is_some())
            && (meta_created.is_some() || meta_deleted.is_some() || meta_updated.is_some())
        {
            return Err(Error::new(
                input.span(),
                format!(
                    "{} can not be a key column and timestamp at the same time",
                    name.to_string()
                ),
            ));
        }

        match (
            primary_key,
            foreign_key,
            unique,
            meta_created,
            meta_updated,
            meta_deleted,
        ) {
            (Some(pk), None, None, None, None, None) => {
                return Ok(Self::PrimaryKey(PrimaryKey { name, ty }))
            }
            (None, Some(fk), None, None, None, None) => {
                return Ok(Self::ForeignKey(ForeignKey {
                    foreign_table: fk.parse_args()?,
                    name,
                    ty,
                }))
            }
            (None, None, _, None, None, None) => {
                return Ok(Self::DataColumn(DataColumn {
                    modifiers: ColumnModifiers { unique: false },
                    name,
                    ty,
                }))
            }
            (None, None, None, Some(_), None, None) => {
                return Ok(Self::MetaColumn(MetaColumn::CreatedAt { name, ty }))
            }
            (None, None, None, None, Some(_), None) => {
                return Ok(Self::MetaColumn(MetaColumn::UpdatedAt { name, ty }))
            }
            (None, None, None, None, None, Some(_)) => {
                return Ok(Self::MetaColumn(MetaColumn::DeletedAt { name, ty }))
            }
            _ => {
                return Err(Error::new(
                    input.span(),
                    format!(
                        "{} has an invalid combination of atmosphere column attributes",
                        name.to_string()
                    ),
                ));
            }
        }
    }
}

impl Column {
    pub fn name(&self) -> &Ident {
        match self {
            Self::PrimaryKey(pk) => &pk.name,
            Self::ForeignKey(fk) => &fk.name,
            Self::DataColumn(data) => &data.name,
            Self::MetaColumn(meta) => &meta.name(),
        }
    }
}

/// Utility implementations for determining the enum type
impl Column {
    pub const fn as_primary_key(&self) -> Option<&PrimaryKey> {
        match self {
            Self::PrimaryKey(pk) => Some(pk),
            _ => None,
        }
    }

    pub const fn as_foreign_key(&self) -> Option<&ForeignKey> {
        match self {
            Self::ForeignKey(fk) => Some(fk),
            _ => None,
        }
    }

    pub const fn as_data_column(&self) -> Option<&DataColumn> {
        match self {
            Self::DataColumn(c) => Some(c),
            _ => None,
        }
    }

    pub const fn as_meta_column(&self) -> Option<&MetaColumn> {
        match self {
            Self::MetaColumn(c) => Some(c),
            _ => None,
        }
    }
}
