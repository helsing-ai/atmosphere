use std::hash::Hash;

use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, quote};
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TimestampKind {
    Created,
    Updated,
    Deleted,
}

impl ToTokens for TimestampKind {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let path = match self {
            Self::Created => quote!(::atmosphere::column::TimestampKind::Created),
            Self::Updated => quote!(::atmosphere::column::TimestampKind::Updated),
            Self::Deleted => quote!(::atmosphere::column::TimestampKind::Deleted),
        };

        tokens.extend(path);
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TimestampColumn {
    pub modifiers: ColumnModifiers,
    pub kind: TimestampKind,
    pub name: NameSet,
    pub ty: Type,
}

impl TimestampColumn {
    pub fn quote(&self) -> TokenStream {
        let kind = self.kind;
        let field = self.name.field().to_string();
        let sql = self.name.sql().to_string();

        quote!(::atmosphere::TimestampColumn::new(
            #kind,
            #field,
            #sql
        ))
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
    Data(DataColumn),
    Timestamp(TimestampColumn),
}

impl Hash for Column {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name().field().to_string().hash(state);
    }
}

impl Column {
    pub fn quote(&self) -> TokenStream {
        match self {
            Self::PrimaryKey(pk) => pk.quote(),
            Self::ForeignKey(fk) => fk.quote(),
            Self::Data(data) => data.quote(),
            Self::Timestamp(time) => time.quote(),
        }
    }

    pub fn ty(&self) -> &syn::Type {
        match self {
            Self::PrimaryKey(pk) => &pk.ty,
            Self::ForeignKey(fk) => &fk.ty,
            Self::Data(data) => &data.ty,
            Self::Timestamp(ts) => &ts.ty,
        }
    }
}

pub mod attribute {
    use syn::{Error, Ident, LitStr, Token, parse::Parse};

    use super::{ColumnModifiers, TimestampKind};

    pub const PATH: &str = "sql";

    const PRIMARY_KEY: &str = "pk";
    const FOREIGN_KEY: &str = "fk";
    const UNIQUE: &str = "unique";
    const TIMESTAMP: &str = "timestamp";

    const TIMESTAMP_CREATED: &str = "created";
    const TIMESTAMP_UPDATED: &str = "updated";
    const TIMESTAMP_DELETED: &str = "deleted";

    #[derive(Clone, Debug, PartialEq, Eq, Hash)]
    pub enum ColumnKind {
        PrimaryKey,
        ForeignKey { on: Ident },
        Data,
        Timestamp { kind: TimestampKind },
    }

    impl Parse for ColumnKind {
        fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
            let mut kind = ColumnKind::Data;

            if let Some((id, _)) = input.cursor().ident() {
                match id.to_string().as_str() {
                    PRIMARY_KEY => {
                        let _: Ident = input.parse()?;

                        kind = ColumnKind::PrimaryKey;
                    }
                    FOREIGN_KEY => {
                        let _: Ident = input.parse()?;

                        input.parse::<Token![-]>()?;
                        input.parse::<Token![>]>()?;

                        let on = input.parse()?;

                        kind = ColumnKind::ForeignKey { on }
                    }
                    TIMESTAMP => {
                        let _: Ident = input.parse()?;

                        input.parse::<Token![=]>()?;

                        let ty: Ident = input.parse()?;

                        let ty = match ty.to_string().as_ref() {
                            TIMESTAMP_CREATED => TimestampKind::Created,
                            TIMESTAMP_UPDATED => TimestampKind::Updated,
                            TIMESTAMP_DELETED => TimestampKind::Deleted,
                            _ => {
                                return Err(syn::Error::new_spanned(
                                    ty,
                                    "`#[sql(timestamp = <type>)]` only supports `created`. `updated` and `deleted`",
                                ));
                            }
                        };

                        kind = ColumnKind::Timestamp { kind: ty }
                    }
                    _ => {}
                };

                if kind != ColumnKind::Data && input.peek(Token![,]) {
                    input.parse::<Token![,]>()?;
                }
            }

            Ok(kind)
        }
    }

    #[derive(Clone, Debug, PartialEq, Eq, Hash)]
    pub struct Attribute {
        pub kind: ColumnKind,
        pub modifiers: ColumnModifiers,
        pub renamed: Option<Ident>,
    }

    impl Parse for Attribute {
        fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
            let kind: ColumnKind = input.parse()?;

            let mut modifiers = ColumnModifiers { unique: false };
            let mut renamed = None;

            while !input.is_empty() {
                let ident: syn::Ident = input.parse()?;

                // we found a tag
                if ident.to_string().as_str() == UNIQUE {
                    if modifiers.unique {
                        return Err(Error::new(
                            ident.span(),
                            "found redundant `unique` modifier",
                        ));
                    }

                    modifiers.unique = true;

                    if !input.peek(Token![,]) {
                        break;
                    }

                    input.parse::<Token![,]>()?;

                    continue;
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
                modifiers,
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
            return Ok(Self::Data(DataColumn {
                modifiers: ColumnModifiers { unique: false },
                name: NameSet::new(name, None),
                ty,
            }));
        };

        let attribute: attribute::Attribute = attribute.parse_args()?;

        let modifiers = attribute.modifiers;
        let name = NameSet::new(name, attribute.renamed);

        match attribute.kind {
            attribute::ColumnKind::PrimaryKey => Ok(Self::PrimaryKey(PrimaryKey {
                modifiers: ColumnModifiers { unique: true },
                name,
                ty,
            })),
            attribute::ColumnKind::ForeignKey { on } => Ok(Self::ForeignKey(ForeignKey {
                on,
                modifiers,
                name,
                ty,
            })),
            attribute::ColumnKind::Data => Ok(Self::Data(DataColumn {
                modifiers,
                name,
                ty,
            })),
            attribute::ColumnKind::Timestamp { kind } => Ok(Self::Timestamp(TimestampColumn {
                modifiers,
                kind,
                name,
                ty,
            })),
        }
    }
}

impl Column {
    pub fn name(&self) -> &NameSet {
        match self {
            Self::PrimaryKey(pk) => &pk.name,
            Self::ForeignKey(fk) => &fk.name,
            Self::Data(data) => &data.name,
            Self::Timestamp(ts) => &ts.name,
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
            Self::Data(c) => Some(c),
            _ => None,
        }
    }

    pub const fn as_timestamp_column(&self) -> Option<&TimestampColumn> {
        match self {
            Self::Timestamp(c) => Some(c),
            _ => None,
        }
    }
}
