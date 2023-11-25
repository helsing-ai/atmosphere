use proc_macro2::TokenStream;
use quote::quote;

use crate::schema::table::Table;

pub fn hooks(table: &Table) -> TokenStream {
    let ident = &table.ident;
    let registered = &table.hooks.registered;

    //let mut derived: Vec<syn::Ident> = vec![];
    //let mut hooks = TokenStream::new();

    //for timestamp in table.timestamp_columns.iter() {
    //let field = timestamp.name.field();

    //let hook = syn::Ident::new(
    //&format!(
    //"__{}TimestampSetter{}",
    //ident.to_string(),
    //field.to_string()
    //),
    //field.span(),
    //);

    //hooks.extend(quote!(
    //struct #hook;

    //#[async_trait::async_trait]
    //impl Hook<#ident> for #hook {
    //fn stage(&self) -> HookStage { HookStage::PreBind }

    //async fn apply(&self, ctx: &Query<#ident>, input: &mut HookInput<'_, #ident>) -> Result<()> {
    //println!(
    //"atmosphere::set::{}.{} because {:?} {:?}",
    //stringify!(#ident), stringify!(#field),
    //ctx.op,
    //ctx.cardinality,
    //);

    //Ok(())
    //}
    //}
    //));

    ////derived.push(hook);
    //}
    //#(&#derived,),*

    quote!(
        #[automatically_derived]
        impl ::atmosphere::hooks::Hooks for #ident {
            const HOOKS: &'static [&'static dyn ::atmosphere::hooks::Hook<#ident>] = &[
                #(&#registered,),*
            ];
        }
    )
    .into()
}
