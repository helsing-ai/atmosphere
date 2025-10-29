use serde::{Deserialize, Serialize};

mod non_nullable;
mod nullable;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Deserialize, Serialize)]
struct Data {
    name: String,
}

impl Data {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
        }
    }
}
