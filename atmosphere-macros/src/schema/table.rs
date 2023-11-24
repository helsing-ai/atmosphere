use std::collections::HashSet;

use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{Error, Fields, Generics, Ident, LitStr, Token, Visibility};

use crate::schema::keys::PrimaryKey;

use super::column::{Column, DataColumn, MetaColumn};
use super::keys::ForeignKey;

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
                    ))
                }
            }

            if !input.peek(Token![,]) {
                break;
            }

            input.parse::<Token![,]>()?;
        }

        let schema = schema.ok_or_else(|| {
            syn::Error::new(input.span(), "`#[table]` requies a value for `schema`")
        })?;

        let table = table.ok_or_else(|| {
            syn::Error::new(input.span(), "`#[table]` requires a value for `name`")
        })?;

        Ok(Self { schema, table })
    }
}

#[derive(Clone, Debug)]
pub struct Table {
    pub vis: Visibility,
    pub generics: Generics,
    pub ident: Ident,

    pub id: TableId,

    pub primary_key: PrimaryKey,

    pub foreign_keys: HashSet<ForeignKey>,
    pub data_columns: HashSet<DataColumn>,
    pub meta_columns: HashSet<MetaColumn>,
}

impl Parse for Table {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let item: syn::ItemStruct = input.parse()?;

        let id: TableId = item
            .attrs
            .iter()
            .find(|attr| attr.path().is_ident("table"))
            .ok_or(syn::Error::new(
                input.span(),
                "You need to use the `#[table]` attribute if you want to derive `Schema`",
            ))?
            .parse_args()?;

        let ident = item.ident;

        let fields = match item.fields {
            Fields::Named(n) => n,
            Fields::Unnamed(_) | Fields::Unit => {
                return Err(Error::new(
                    ident.span(),
                    format!(
                        "{} must use named fields in order to derive `Schema`",
                        ident.to_string()
                    ),
                ))
            }
        };

        let columns = fields
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
                    input.span(),
                    format!(
                        "{} declares more than one column as its primary key – only one is allowed",
                        ident.to_string()
                    ),
                ));
            }

            primary_keys.into_iter().next().ok_or(Error::new(
                input.span(),
                format!(
                    "{} must declare one field as its primary key (using `#[primary_key]`",
                    ident.to_string()
                ),
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

        let meta_columns = columns
            .iter()
            .filter_map(|c| c.as_meta_column())
            .cloned()
            .collect();

        Ok(Self {
            vis: item.vis,
            generics: item.generics,
            ident,
            id,
            primary_key,
            foreign_keys,
            data_columns,
            meta_columns,
        })
    }
}

impl Table {
    pub fn quote_table_impl(&self) -> TokenStream {
        let Self {
            ident,
            id,
            primary_key,
            foreign_keys,
            data_columns,
            meta_columns,
            ..
        } = self;

        let schema = id.schema.to_string();
        let table = id.table.to_string();

        let pk_ty = &self.primary_key.ty;
        let pk_field = &self.primary_key.name.field();

        let primary_key = primary_key.quote();
        let foreign_keys = foreign_keys.iter().map(|r| r.quote());
        let data = data_columns.iter().map(|d| d.quote());
        let meta = meta_columns.iter().map(|d| d.quote());

        quote!(
            #[automatically_derived]
            impl ::atmosphere::Table for #ident {
                type PrimaryKey = #pk_ty;

                const SCHEMA: &'static str = #schema;
                const TABLE: &'static str = #table;

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

    pub fn quote_field_query_impls(&self) -> TokenStream {
        let mut stream = TokenStream::new();

        let ident = &self.ident;

        let fks: Vec<Column> = self
            .foreign_keys
            .iter()
            .filter(|fk| fk.modifiers.unique)
            .cloned()
            .map(|fk| Column::ForeignKey(fk))
            .collect();

        let data: Vec<Column> = self
            .data_columns
            .iter()
            .filter(|data| data.modifiers.unique)
            .cloned()
            .map(|data| Column::DataColumn(data))
            .collect();

        dbg!(&self.data_columns);
        dbg!(&data);

        for column in fks.iter().chain(data.iter()) {
            let ty = column.ty();
            let col = column.name().field().to_string().to_lowercase();
            let column = column.quote();

            let find_by_col = Ident::new(&format!("find_by_{col}"), Span::mixed_site());
            let delete_by_col = Ident::new(&format!("delete_by_{col}"), Span::mixed_site());

            stream.extend(quote!(
                #[automatically_derived]
                impl #ident {
                    pub async fn #find_by_col<'e, E>(
                        value: &#ty,
                        executor: E
                    ) -> ::atmosphere::Result<Option<#ident>>
                    where
                        E: ::sqlx::Executor<'e, Database = ::atmosphere::Driver>,
                        for<'q> <::atmosphere::Driver as ::sqlx::database::HasArguments<'q>>::Arguments:
                            ::sqlx::IntoArguments<'q, ::atmosphere::Driver> + Send
                    {
                        use ::atmosphere::{
                            query::{Query, QueryError},
                            runtime::sql,
                            Error
                        };

                        static COLUMN: &'static ::atmosphere::Column<#ident> = &#column.as_col();

                        let query = sql::select_by::<#ident>(COLUMN.to_owned());

                        dbg!(query.sql());

                        ::sqlx::query_as(query.sql())
                            .bind(value)
                            .persistent(false)
                            .fetch_optional(executor)
                            .await
                            .map_err(QueryError::from)
                            .map_err(Error::Query)
                    }

                    pub async fn #delete_by_col<'e, E>(
                        value: &#ty,
                        executor: E,
                    ) -> ::atmosphere::Result<<::atmosphere::Driver as ::sqlx::Database>::QueryResult>
                    where
                        E: ::sqlx::Executor<'e, Database = ::atmosphere::Driver>,
                        for<'q> <::atmosphere::Driver as ::sqlx::database::HasArguments<'q>>::Arguments:
                            ::sqlx::IntoArguments<'q, ::atmosphere::Driver> + Send
                    {
                        use ::atmosphere::{
                            query::{Query, QueryError},
                            runtime::sql,
                            Error
                        };

                        static COLUMN: &'static ::atmosphere::Column<#ident> = &#column.as_col();

                        let query = sql::delete_by::<#ident>(COLUMN.to_owned());
                        dbg!(query.sql());

                        ::sqlx::query(query.sql())
                            .bind(value)
                            .persistent(false)
                            .execute(executor)
                            .await
                            .map_err(QueryError::from)
                            .map_err(Error::Query)
                    }
                }
            ))
        }

        dbg!(stream.clone().to_string());

        stream
    }

    pub fn quote_rel_impls(&self) -> TokenStream {
        let mut stream = TokenStream::new();

        let ident = &self.ident;

        for fk in self.foreign_keys.iter() {
            let col = fk.quote();

            let other = &fk.on;

            let find_all_self = Ident::new(
                &format!("{}s", ident.to_string().to_lowercase()),
                Span::mixed_site(),
            );

            let find_other = Ident::new(
                &format!("{}", other.to_string().to_lowercase()),
                Span::mixed_site(),
            );

            let find_by_other = Ident::new(
                &format!("find_by_{}", other.to_string().to_lowercase()),
                Span::mixed_site(),
            );

            let delete_self = Ident::new(
                &format!("delete_{}s", ident.to_string().to_lowercase()),
                Span::mixed_site(),
            );

            stream.extend(quote!(
                #[automatically_derived]
                impl #ident {
                    pub async fn #find_other<'e, E>(
                        &self,
                        executor: E,
                    ) -> ::atmosphere::Result<#other>
                    where
                        E: ::sqlx::Executor<'e, Database = ::atmosphere::Driver>,
                        for<'q> <::atmosphere::Driver as ::sqlx::database::HasArguments<'q>>::Arguments:
                            ::sqlx::IntoArguments<'q, ::atmosphere::Driver> + Send {
                        <#ident as ::atmosphere::rel::RefersTo<#other>>::resolve(&self, executor).await
                    }

                    pub async fn #find_by_other<'e, E>(
                        pk: &<#other as ::atmosphere::Table>::PrimaryKey,
                        executor: E,
                        // TODO: either Vec<Self>, or if marked as unique, only Self
                    ) -> ::atmosphere::Result<Vec<#ident>>
                    where
                        E: ::sqlx::Executor<'e, Database = ::atmosphere::Driver>,
                        for<'q> <::atmosphere::Driver as ::sqlx::database::HasArguments<'q>>::Arguments:
                            ::sqlx::IntoArguments<'q, ::atmosphere::Driver> + Send {
                        <#other as ::atmosphere::rel::ReferedBy<#ident>>::resolve_by(pk, executor).await
                    }
                }

                #[automatically_derived]
                impl #other {
                    pub async fn #find_all_self<'e, E>(
                        &self,
                        executor: E,
                    ) -> ::atmosphere::Result<Vec<#ident>>
                    where
                        E: ::sqlx::Executor<'e, Database = ::atmosphere::Driver>,
                        for<'q> <::atmosphere::Driver as ::sqlx::database::HasArguments<'q>>::Arguments:
                            ::sqlx::IntoArguments<'q, ::atmosphere::Driver> + Send {
                        <#other as ::atmosphere::rel::ReferedBy<#ident>>::resolve(&self, executor).await
                    }

                    pub async fn #delete_self<'e, E>(
                        &self,
                        executor: E,
                    ) -> ::atmosphere::Result<<::atmosphere::Driver as ::sqlx::Database>::QueryResult>
                    where
                        E: ::sqlx::Executor<'e, Database = ::atmosphere::Driver>,
                        for<'q> <::atmosphere::Driver as ::sqlx::database::HasArguments<'q>>::Arguments:
                            ::sqlx::IntoArguments<'q, ::atmosphere::Driver> + Send {
                        <#other as ::atmosphere::rel::ReferedBy<#ident>>::delete_all(&self, executor).await
                    }
                }

                #[automatically_derived]
                impl ::atmosphere::rel::RefersTo<#other> for #ident {
                    const FOREIGN_KEY: ::atmosphere::ForeignKey<#ident> = #col;
                }

                #[automatically_derived]
                impl ::atmosphere::rel::ReferedBy<#ident> for #other {}
            ));
        }

        stream
    }

    pub fn quote_bind_impl(&self) -> TokenStream {
        let col = Ident::new("col", proc_macro2::Span::call_site());
        let query = Ident::new("query", proc_macro2::Span::call_site());

        let mut binds = TokenStream::new();

        {
            let field = &self.primary_key.name.field();

            binds.extend(quote!(
                if #col.field() == Self::PRIMARY_KEY.field {
                    use ::atmosphere::Bindable;
                    return Ok(#query.dyn_bind(&self.#field));
                }
            ));
        }

        for ref fk in &self.foreign_keys {
            let field = fk.name.field();

            binds.extend(quote!(
                if #col.field() == stringify!(#field) {
                    use ::atmosphere::Bindable;
                    return Ok(#query.dyn_bind(&self.#field));
                }
            ));
        }

        for ref data in &self.data_columns {
            let field = data.name.field();

            binds.extend(quote!(
                if #col.field() == stringify!(#field) {
                    use ::atmosphere::Bindable;
                    return Ok(#query.dyn_bind(&self.#field));
                }
            ));
        }

        let ident = &self.ident;

        quote!(
            #[automatically_derived]
            impl ::atmosphere::Bind for #ident {
                fn bind<
                    'q,
                    Q: ::atmosphere::Bindable<'q>
                >(
                    &'q self,
                    #col: &'q ::atmosphere::Column<Self>,
                    #query: Q
                ) -> ::atmosphere::Result<Q> {
                    #binds

                    Err(::atmosphere::Error::Bind(
                        ::atmosphere::bind::BindError::Unknown(#col.field())
                    ))
                }
            }
        )
    }
}
