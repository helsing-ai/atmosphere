use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::Ident;

use crate::schema::table::Table;

pub fn relationships(table: &Table) -> TokenStream {
    let mut stream = TokenStream::new();

    let ident = &table.ident;

    for fk in table.foreign_keys.iter() {
        let col = fk.quote();

        let other = &fk.on;

        let find_all_self = Ident::new(
            &format!("{}s", ident.to_string().to_lowercase()),
            Span::mixed_site(),
        );

        let find_other = Ident::new(
            &fk.name.field().to_string().to_lowercase().to_string(),
            Span::mixed_site(),
        );

        let find_by_other = Ident::new(
            &format!("find_by_{}", fk.name.field().to_string().to_lowercase()),
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
                    <#other as ::atmosphere::rel::ReferredBy<#ident>>::resolve_by(pk, executor).await
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
                    <#other as ::atmosphere::rel::ReferredBy<#ident>>::resolve(&self, executor).await
                }

                pub async fn #delete_self<'e, E>(
                    &self,
                    executor: E,
                ) -> ::atmosphere::Result<<::atmosphere::Driver as ::sqlx::Database>::QueryResult>
                where
                    E: ::sqlx::Executor<'e, Database = ::atmosphere::Driver>,
                    for<'q> <::atmosphere::Driver as ::sqlx::database::HasArguments<'q>>::Arguments:
                        ::sqlx::IntoArguments<'q, ::atmosphere::Driver> + Send {
                    <#other as ::atmosphere::rel::ReferredBy<#ident>>::delete_all(&self, executor).await
                }
            }

            #[automatically_derived]
            impl ::atmosphere::rel::RefersTo<#other> for #ident {
                const FOREIGN_KEY: ::atmosphere::ForeignKey<#ident> = #col;
            }

            #[automatically_derived]
            impl ::atmosphere::rel::ReferredBy<#ident> for #other {}
        ));
    }

    stream
}
