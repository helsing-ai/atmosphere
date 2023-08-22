use lazy_static::lazy_static;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;
use std::sync::Mutex;

use crate::table::Table;

pub type QualifiedTableID = (Schema, TableID);

/// The id of a table
pub type TableID = String;

/// A database schema
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Schema {
    Public,
    Custom(String),
}

impl fmt::Display for Schema {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Public => f.write_str("public"),
            Self::Custom(c) => f.write_str(c.as_str()),
        }
    }
}
