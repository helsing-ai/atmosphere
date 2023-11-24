use std::hash::Hash;

use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Field, Ident, Type};

use super::keys::{ForeignKey, PrimaryKey};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct NameSet {
    field: Ident,
    sql: Option<Ident>,
}

impl NameSet {
    pub fn new(field: Ident, sql: Option<Ident>) -> Self {
        Self { field, sql }
    }

    pub fn field(&self) -> &Ident {
        &self.field
    }

    pub fn sql(&self) -> &Ident {
        self.sql.as_ref().unwrap_or(&self.field)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ColumnModifiers {
    pub unique: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum MetaColumn {
    CreatedAt { name: NameSet, ty: Type },
    UpdatedAt { name: NameSet, ty: Type },
    DeletedAt { name: NameSet, ty: Type },
}

impl MetaColumn {
    pub fn name(&self) -> &NameSet {
        match self {
            Self::CreatedAt { name, .. }
            | Self::UpdatedAt { name, .. }
            | Self::DeletedAt { name, .. } => name,
        }
    }

    pub fn quote(&self) -> TokenStream {
        //let name = self.name().to_string();

        unimplemented!()

        //quote!(::atmosphere::DataColumn::new(
        //stringify!(#field),
        //stringify!(#sql)
        //))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct DataColumn {
    pub modifiers: ColumnModifiers,
    pub name: NameSet,
    pub ty: Type,
}

impl DataColumn {
    pub fn quote(&self) -> TokenStream {
        let field = self.name.field();
        let sql = self.name.sql();

        quote!(::atmosphere::DataColumn::new(
            stringify!(#field),
            stringify!(#sql)
        ))
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
        self.name().field().to_string().hash(state);
    }
}

pub mod attribute {
    use syn::{parse::Parse, Error, Ident, LitStr, Token};

    use super::ColumnModifiers;

    pub const PATH: &str = "sql";

    const PRIMARY_KEY: &str = "pk";
    const FOREIGN_KEY: &str = "fk";
    const UNIQUE: &str = "unique";

    const META_CREATED_AT: &str = "created_at";
    const META_UPDATED_AT: &str = "updated_at";
    const META_DELETED_AT: &str = "deleted_at";

    #[derive(Clone, Debug, PartialEq, Eq, Hash)]
    pub struct Uniqueness;

    #[derive(Clone, Debug, PartialEq, Eq, Hash)]
    pub enum ColumnKind {
        PrimaryKey,
        ForeignKey { on: Ident },
        Data,
    }

    impl Parse for ColumnKind {
        fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
            let ident: Ident = input.parse()?;

            let kind = match ident.to_string().as_str() {
                PRIMARY_KEY => ColumnKind::PrimaryKey,
                FOREIGN_KEY => {
                    input.parse::<Token![-]>()?;
                    input.parse::<Token![>]>()?;

                    let on = input.parse()?;

                    ColumnKind::ForeignKey { on }
                }
                _ => ColumnKind::Data,
            };

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }

            Ok(kind)
        }
    }

    #[derive(Clone, Debug, PartialEq, Eq, Hash)]
    pub struct Attribute {
        pub kind: ColumnKind,
        pub modifers: ColumnModifiers,
        pub renamed: Option<Ident>,
    }

    impl Parse for Attribute {
        fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
            let kind: ColumnKind = input.parse()?;

            let mut modifers = ColumnModifiers { unique: false };
            let mut renamed = None;

            while !input.is_empty() {
                let ident: syn::Ident = input.parse()?;

                // we found a tag
                if input.peek(Token![,]) {
                    if ident.to_string().as_str() == UNIQUE {
                        if modifers.unique == true {
                            return Err(Error::new(
                                ident.span(),
                                "found redundant `unique` modifier",
                            ));
                        }

                        modifers.unique = true;
                    }
                }

                // we found a kv pair
                input.parse::<Token![=]>()?;
                let value: LitStr = input.parse()?;

                match ident.to_string().as_str() {
                    "rename" => renamed = Some(Ident::new(&value.value(), value.span())),
                    _ => return Err(syn::Error::new_spanned(ident, "")),
                }

                if !input.peek(Token![,]) {
                    break;
                }

                input.parse::<Token![,]>()?;
            }

            Ok(Self {
                kind,
                modifers,
                renamed,
            })
        }
    }
}

impl TryFrom<Field> for Column {
    type Error = syn::Error;

    fn try_from(field: Field) -> syn::Result<Self> {
        let name = field.ident.ok_or(syn::Error::new(
            Span::call_site(),
            "only named fields are supported",
        ))?;

        let ty = field.ty;

        let attribute = field
            .attrs
            .iter()
            .find(|a| a.path().is_ident(attribute::PATH));

        let Some(attribute) = attribute else {
            return Ok(Self::DataColumn(DataColumn {
                modifiers: ColumnModifiers { unique: false },
                name: NameSet::new(name, None),
                ty,
            }));
        };

        let attribute: attribute::Attribute = attribute.parse_args()?;

        return match attribute.kind {
            attribute::ColumnKind::PrimaryKey => Ok(Self::PrimaryKey(PrimaryKey {
                modifiers: ColumnModifiers {
                    unique: true,
                    ..attribute.modifers
                },
                name: NameSet::new(name, attribute.renamed),
                ty,
            })),
            attribute::ColumnKind::ForeignKey { on } => Ok(Self::ForeignKey(ForeignKey {
                on,
                modifiers: attribute.modifers,
                name: NameSet::new(name, attribute.renamed),
                ty,
            })),
            attribute::ColumnKind::Data => Ok(Self::DataColumn(DataColumn {
                modifiers: attribute.modifers,
                name: NameSet::new(name, attribute.renamed),
                ty,
            })),
        };
    }
}

impl Column {
    pub fn name(&self) -> &NameSet {
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
