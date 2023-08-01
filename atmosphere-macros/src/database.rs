use std::collections::HashMap;
use std::sync::Mutex;

use proc_macro::{self, Span, TokenStream};
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote, ToTokens};
use sqlx::{Postgres, QueryBuilder};
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{
    parse_macro_input, parse_quote, Attribute, Data, DataStruct, DeriveInput, Expr, ExprLit, Field,
    Fields, FieldsNamed, Ident, Lifetime, Lit, LitStr, Meta, MetaNameValue, Stmt,
};

use crate::table::Table;

/// A global map for inspecting the database
pub struct Database;

pub type QualifiedTableID = (Schema, TableID);

/// The id of a table
pub type TableID = String;

/// A database schema
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Schema {
    Default,
    Custom(String),
}

type SharedTableMap = Mutex<HashMap<QualifiedTableID, Table>>;

impl std::ops::Deref for Database {
    type Target = SharedTableMap;

    fn deref(&self) -> &Self::Target {
        static ONCE: std::sync::Once = std::sync::Once::new();
        static mut VALUE: *mut SharedTableMap = 0 as *mut SharedTableMap;

        unsafe {
            ONCE.call_once(|| VALUE = Box::into_raw(Box::new(Mutex::new(HashMap::new()))));
            &*VALUE
        }
    }
}
